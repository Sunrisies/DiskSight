pub mod dir_listing;
pub mod dir_listing_v2;
pub mod models;
pub mod utils;
use std::path::Path;

pub use dir_listing::*;
pub use dir_listing_v2::*;
pub use models::*;
use tauri::async_runtime::spawn_blocking;
use tauri::AppHandle;
use tauri::Emitter;
pub use utils::*;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn calculate_dir_size_simple_fast(path: String) -> Result<DirectoryResult, String> {
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

    let start_time = std::time::Instant::now();
    let result = spawn_blocking(move || match list_directory(Path::new(&path), &cli) {
        Ok(entries) => {
            let elapsed = start_time.elapsed().as_secs_f64();
            Ok(DirectoryResult {
                entries,
                query_time: elapsed,
            })
        }
        Err(e) => Err(format!("Error listing directory: {}", e)),
    })
    .await
    .map_err(|e| format!("Failed to execute blocking task: {}", e))?;

    result
}
// 发送进度事件的辅助函数
fn emit_progress(app_handle: &AppHandle, current_path: &Path, current_file: &Path, status: &str) {
    let _ = app_handle.emit(
        "scan-progress",
        ProgressEvent {
            current_path: current_path.to_string_lossy().to_string(),
            current_file: current_file.to_string_lossy().to_string(),
            status: status.to_string(),
        },
    );
}

#[tauri::command]
async fn get_list_directory(
    path: String,

    app_handle: AppHandle,
) -> Result<DirectoryResult, String> {
    let start_time = std::time::Instant::now();

    // 在闭包前克隆 app_handle
    let app_handle_clone = app_handle.clone();
    // 发送开始事件
    let _ = app_handle.emit("scan-started", ());

    let result = spawn_blocking(move || {
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

        // 修改 list_directory 以接受进度回调
        list_directory_with_events(Path::new(&path), &cli, &app_handle)
    })
    .await
    .map_err(|e| format!("Failed to execute blocking task: {}", e))?;

    match result {
        Ok(entries) => {
            let _ = app_handle_clone.emit("scan-completed", ());
            let elapsed = start_time.elapsed().as_secs_f64();
            Ok(DirectoryResult {
                entries,
                query_time: elapsed,
            })
        }
        Err(e) => {
            let _ = app_handle_clone.emit("scan-error", e.to_string());
            Err(format!("Error listing directory: {}", e))
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_list_directory,
            calculate_dir_size_simple_fast
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
