// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--steam-worker" {
        slaymumanager_lib::handle_steam_worker(&args[2..]);
        return;
    }
    slaymumanager_lib::run()
}
