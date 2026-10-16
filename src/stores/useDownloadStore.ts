import { defineStore } from "pinia"
import { ref, computed } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { listen, type UnlistenFn } from "@tauri-apps/api/event"
import type { DownloadTask } from "@/types"

export const useDownloadStore = defineStore("download", () => {
  const tasks = ref<DownloadTask[]>([])
  const showDrawer = ref(false)
  let unlistenProgress: UnlistenFn | null = null
  let unlistenFinished: UnlistenFn | null = null

  const activeDownloads = computed(() =>
    tasks.value.filter(t => t.status === "downloading" || t.status === "pending")
  )
  const completedDownloads = computed(() =>
    tasks.value.filter(t => t.status === "completed")
  )
  const failedDownloads = computed(() =>
    tasks.value.filter(t => t.status === "failed")
  )
  const totalActiveSpeed = computed(() =>
    activeDownloads.value.reduce((acc, t) => acc + (t.speedBytesPerSec || 0), 0)
  )

  async function fetchTasks() {
    try {
      const list = await invoke<DownloadTask[]>("get_download_tasks")
      tasks.value = list
    } catch {
      // silent
    }
  }

  async function startNexusDownload(modId: number, fileId: number, modName: string, fileName: string) {
    const task = await invoke<DownloadTask>("start_nexus_download", {
      modId,
      fileId,
      modName,
      fileName,
    })
    const idx = tasks.value.findIndex(t => t.id === task.id)
    if (idx >= 0) {
      tasks.value[idx] = task
    } else {
      tasks.value.unshift(task)
    }
    return task
  }

  async function pauseDownload(taskId: string) {
    await invoke("pause_download", { taskId })
    const task = tasks.value.find(t => t.id === taskId)
    if (task) {
      task.status = "paused"
      task.speedBytesPerSec = 0
    }
  }

  async function resumeDownload(taskId: string) {
    await invoke("resume_download", { taskId })
    const task = tasks.value.find(t => t.id === taskId)
    if (task) {
      task.status = "pending"
    }
  }

  async function cancelDownload(taskId: string) {
    await invoke("cancel_download", { taskId })
    const task = tasks.value.find(t => t.id === taskId)
    if (task) {
      task.status = "cancelled"
      task.speedBytesPerSec = 0
    }
  }

  async function clearCompleted() {
    await invoke("clear_completed_downloads")
    await fetchTasks()
  }

  async function openDownloadsFolder() {
    await invoke("open_downloads_folder")
  }

  async function initListeners() {
    if (unlistenProgress || unlistenFinished) return
    await fetchTasks()

    unlistenProgress = await listen<DownloadTask>("download-progress", (event) => {
      const updated = event.payload
      const idx = tasks.value.findIndex(t => t.id === updated.id)
      if (idx >= 0) {
        tasks.value[idx] = updated
      } else {
        tasks.value.unshift(updated)
      }
    })

    unlistenFinished = await listen<DownloadTask>("download-finished", (event) => {
      const updated = event.payload
      const idx = tasks.value.findIndex(t => t.id === updated.id)
      if (idx >= 0) {
        tasks.value[idx] = updated
      } else {
        tasks.value.unshift(updated)
      }
    })
  }

  function cleanupListeners() {
    unlistenProgress?.()
    unlistenFinished?.()
    unlistenProgress = null
    unlistenFinished = null
  }

  return {
    tasks,
    showDrawer,
    activeDownloads,
    completedDownloads,
    failedDownloads,
    totalActiveSpeed,
    fetchTasks,
    startNexusDownload,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    clearCompleted,
    openDownloadsFolder,
    initListeners,
    cleanupListeners,
  }
})
