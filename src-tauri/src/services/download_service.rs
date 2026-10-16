use crate::utils::error::AppError;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: String,
    pub mod_id: u32,
    pub file_id: u64,
    pub mod_name: String,
    pub file_name: String,
    pub download_url: String,
    pub save_path: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub speed_bytes_per_sec: u64,
    pub status: DownloadStatus,
    pub error_message: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub installed: bool,
    pub install_error: Option<String>,
}

static TASKS: LazyLock<Mutex<Vec<DownloadTask>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static CONTROLS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn get_download_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("slaymumanager").join("downloads");
    std::fs::create_dir_all(&dir).ok();
    dir
}

fn sanitize_file_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect();
    if sanitized.is_empty() {
        "mod_download.zip".to_string()
    } else {
        sanitized
    }
}

pub fn get_download_tasks() -> Vec<DownloadTask> {
    let tasks = TASKS.lock();
    tasks.clone()
}

pub fn clear_completed_tasks() -> Result<(), AppError> {
    let mut tasks = TASKS.lock();
    tasks.retain(|t| t.status == DownloadStatus::Downloading || t.status == DownloadStatus::Pending);
    Ok(())
}

pub fn start_download(
    app_handle: AppHandle,
    mod_id: u32,
    file_id: u64,
    mod_name: String,
    file_name: String,
    download_url: String,
) -> Result<DownloadTask, AppError> {
    let task_id = Uuid::new_v4().to_string();
    let clean_file_name = sanitize_file_name(&file_name);
    let download_dir = get_download_dir();
    let save_path = download_dir.join(&clean_file_name);

    let task = DownloadTask {
        id: task_id.clone(),
        mod_id,
        file_id,
        mod_name,
        file_name: clean_file_name,
        download_url,
        save_path: save_path.to_string_lossy().to_string(),
        total_bytes: 0,
        downloaded_bytes: 0,
        speed_bytes_per_sec: 0,
        status: DownloadStatus::Pending,
        error_message: None,
        created_at: chrono::Local::now().to_rfc3339(),
        completed_at: None,
        installed: false,
        install_error: None,
    };

    {
        let mut tasks = TASKS.lock();
        tasks.push(task.clone());
    }

    spawn_download_thread(app_handle, task_id.clone());
    Ok(task)
}

pub fn pause_download(task_id: &str) -> Result<(), AppError> {
    if let Some(flag) = CONTROLS.lock().get(task_id) {
        flag.store(true, Ordering::SeqCst);
    }
    let mut tasks = TASKS.lock();
    if let Some(t) = tasks.iter_mut().find(|t| t.id == task_id)
        && (t.status == DownloadStatus::Downloading || t.status == DownloadStatus::Pending) {
            t.status = DownloadStatus::Paused;
            t.speed_bytes_per_sec = 0;
    }
    Ok(())
}

pub fn resume_download(app_handle: AppHandle, task_id: &str) -> Result<(), AppError> {
    {
        let mut tasks = TASKS.lock();
        if let Some(t) = tasks.iter_mut().find(|t| t.id == task_id)
            && (t.status == DownloadStatus::Paused || t.status == DownloadStatus::Failed) {
                t.status = DownloadStatus::Pending;
                t.error_message = None;
        }
    }
    spawn_download_thread(app_handle, task_id.to_string());
    Ok(())
}

pub fn cancel_download(task_id: &str) -> Result<(), AppError> {
    if let Some(flag) = CONTROLS.lock().get(task_id) {
        flag.store(true, Ordering::SeqCst);
    }
    let mut tasks = TASKS.lock();
    if let Some(t) = tasks.iter_mut().find(|t| t.id == task_id) {
        t.status = DownloadStatus::Cancelled;
        t.speed_bytes_per_sec = 0;
        let part_file = PathBuf::from(format!("{}.part", t.save_path));
        let _ = std::fs::remove_file(part_file);
    }
    Ok(())
}

fn spawn_download_thread(app_handle: AppHandle, task_id: String) {
    let stop_flag = Arc::new(AtomicBool::new(false));
    {
        let mut controls = CONTROLS.lock();
        controls.insert(task_id.clone(), stop_flag.clone());
    }

    std::thread::spawn(move || {
        let task_data = {
            let tasks = TASKS.lock();
            tasks.iter().find(|t| t.id == task_id).cloned()
        };

        let mut task = match task_data {
            Some(t) => t,
            None => return,
        };

        task.status = DownloadStatus::Downloading;
        update_and_emit_task(&app_handle, &task);

        let part_path = PathBuf::from(format!("{}.part", task.save_path));
        let initial_bytes = if part_path.exists() {
            std::fs::metadata(&part_path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        let client = match reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(3600))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                task.status = DownloadStatus::Failed;
                task.error_message = Some(format!("HTTP 客户端初始化失败: {}", e));
                update_and_emit_task(&app_handle, &task);
                return;
            }
        };

        let mut req = client.get(&task.download_url);
        if initial_bytes > 0 {
            req = req.header("Range", format!("bytes={}-", initial_bytes));
        }

        let mut resp = match req.send() {
            Ok(r) => r,
            Err(e) => {
                task.status = DownloadStatus::Failed;
                task.error_message = Some(format!("连接下载服务器失败: {}", e));
                update_and_emit_task(&app_handle, &task);
                return;
            }
        };

        if !resp.status().is_success() && resp.status().as_u16() != 206 {
            task.status = DownloadStatus::Failed;
            task.error_message = Some(format!("下载服务响应异常: HTTP {}", resp.status()));
            update_and_emit_task(&app_handle, &task);
            return;
        }

        let content_length = resp.content_length().unwrap_or(0);
        task.total_bytes = if resp.status().as_u16() == 206 {
            initial_bytes + content_length
        } else {
            content_length
        };
        task.downloaded_bytes = initial_bytes;

        let mut file = match OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&part_path)
        {
            Ok(f) => f,
            Err(e) => {
                task.status = DownloadStatus::Failed;
                task.error_message = Some(format!("创建临时文件失败: {}", e));
                update_and_emit_task(&app_handle, &task);
                return;
            }
        };

        if initial_bytes > 0 && resp.status().as_u16() == 206 {
            let _ = file.seek(SeekFrom::Start(initial_bytes));
        } else if initial_bytes > 0 && resp.status().as_u16() != 206 {
            let _ = file.set_len(0);
            task.downloaded_bytes = 0;
        }

        let mut buffer = [0u8; 16384];
        let mut last_emit = Instant::now();
        let mut last_bytes = task.downloaded_bytes;
        let mut speed_calc_time = Instant::now();

        loop {
            if stop_flag.load(Ordering::SeqCst) {
                return;
            }

            match resp.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    if let Err(e) = file.write_all(&buffer[..n]) {
                        task.status = DownloadStatus::Failed;
                        task.error_message = Some(format!("写入磁盘失败: {}", e));
                        update_and_emit_task(&app_handle, &task);
                        return;
                    }
                    task.downloaded_bytes += n as u64;

                    let now = Instant::now();
                    if now.duration_since(speed_calc_time) >= Duration::from_millis(500) {
                        let bytes_diff = task.downloaded_bytes.saturating_sub(last_bytes);
                        let secs = now.duration_since(speed_calc_time).as_secs_f64();
                        task.speed_bytes_per_sec = if secs > 0.0 {
                            (bytes_diff as f64 / secs) as u64
                        } else {
                            0
                        };
                        last_bytes = task.downloaded_bytes;
                        speed_calc_time = now;
                    }

                    if now.duration_since(last_emit) >= Duration::from_millis(250) {
                        update_and_emit_task(&app_handle, &task);
                        last_emit = now;
                    }
                }
                Err(e) => {
                    task.status = DownloadStatus::Failed;
                    task.error_message = Some(format!("下载网络中断: {}", e));
                    update_and_emit_task(&app_handle, &task);
                    return;
                }
            }
        }

        drop(file);

        let final_path = PathBuf::from(&task.save_path);
        if final_path.exists() {
            let _ = std::fs::remove_file(&final_path);
        }
        if let Err(e) = std::fs::rename(&part_path, &final_path) {
            task.status = DownloadStatus::Failed;
            task.error_message = Some(format!("重命名下载文件失败: {}", e));
            update_and_emit_task(&app_handle, &task);
            return;
        }

        task.status = DownloadStatus::Completed;
        task.speed_bytes_per_sec = 0;
        task.completed_at = Some(chrono::Local::now().to_rfc3339());

        // 尝试自动解压并安装到游戏 mods 目录
        if let Some(settings) = crate::repositories::settings_repo::load_settings()
            && let Some(game_dir) = settings.game_root_dir
            && !game_dir.is_empty() {
                let game_path = Path::new(&game_dir);
                if game_path.exists() {
                    match crate::workflows::install_archive_workflow::execute_install(
                        &final_path,
                        game_path,
                        true,
                        true,
                    ) {
                        Ok(_) => {
                            task.installed = true;
                        }
                        Err(e) => {
                            task.install_error = Some(format!("模组解压安装未完成: {}", e));
                        }
                    }
                }
        }

        update_and_emit_task(&app_handle, &task);
        let _ = app_handle.emit("download-finished", &task);
    });
}

fn update_and_emit_task(app_handle: &AppHandle, task: &DownloadTask) {
    {
        let mut tasks = TASKS.lock();
        if let Some(t) = tasks.iter_mut().find(|t| t.id == task.id) {
            *t = task.clone();
        }
    }
    let _ = app_handle.emit("download-progress", task);
}
