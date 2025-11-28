pub mod dir_listing;
pub mod models;
pub mod utils;

use std::path::Path;

pub use dir_listing::*;
pub use models::*;
pub use utils::*;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[tauri::command]
fn get_list_directory(path: &str) -> Result<Vec<FileEntry>, String> {
    let cli = Cli {
        file: None,
        long_format: true,
        human_readable: true,
        all: true,
        show_time: true,
        parallel: true,
        sort: true,
        name: None,
        full_path: true,
    };

    match list_directory(Path::new(path), &cli) {
        Ok(entries) => Ok(entries),
        Err(e) => Err(format!("Error listing directory: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_list_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
