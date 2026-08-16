pub mod dir_listing;
pub mod dir_listing_v2;
pub mod models;
pub mod utils;
pub use dir_listing::*;
pub use dir_listing_v2::*;
pub use models::*;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::async_runtime::spawn;
use tauri::async_runtime::spawn_blocking;
use tauri::Emitter;
use tauri::{AppHandle, Manager, State};

use tokio::time::{sleep, Duration};
pub use utils::*;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn calculate_dir_size_simple_fast(
    path: String,
    parallel: bool,
    sort: bool,
    human_readable: bool,
    show_hidden_files: bool,
    state: State<'_, ScanState>,
) -> Result<DirectoryResult, String> {
    let cli = Cli {
        file: None,
        long_format: true,
        human_readable,
        all: true,
        show_time: true,
        parallel,
        sort,
        name: None,
        full_path: true,
    };

    let abort = state.abort.clone();
    let start_time = std::time::Instant::now();
    let result = spawn_blocking(move || match list_directory(Path::new(&path), &cli, abort, show_hidden_files) {
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
    parallel: bool,
    sort: bool,
    human_readable: bool,
    show_hidden_files: bool,
    app_handle: AppHandle,
    state: State<'_, ScanState>,
) -> Result<DirectoryResult, String> {
    let start_time = std::time::Instant::now();
    let app_handle_clone = app_handle.clone();
    let _ = app_handle.emit("scan-started", ());

    let abort = state.abort.clone();
    state.abort.store(false, Ordering::SeqCst);

    let app_handle_inner = app_handle_clone.clone();
    let result = spawn_blocking(move || {
        let cli = Cli {
            file: None,
            long_format: true,
            human_readable,
            all: true,
            show_time: true,
            parallel,
            sort,
            name: None,
            full_path: true,
        };

        list_directory_with_events(Path::new(&path), &cli, &app_handle_inner, abort, show_hidden_files)
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

#[tauri::command]
async fn cancel_scan(state: State<'_, ScanState>) -> Result<(), String> {
    state.abort.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
async fn delete_file(path: String, force: bool) -> Result<(), String> {
    let path = Path::new(&path);

    if !path.exists() {
        return Err("路径不存在".to_string());
    }

    match fs::metadata(&path) {
        Ok(metadata) => {
            if metadata.permissions().readonly() {
                if !force {
                    return Err("路径是只读的。如要强制删除，请设置 force 参数为 true".to_string());
                }
                let mut perms = metadata.permissions();
                perms.set_readonly(false);
                if let Err(e) = fs::set_permissions(path, perms) {
                    return Err(format!("无法修改路径权限: {}", e));
                }
            }
        }
        Err(e) => return Err(format!("无法访问路径: {}", e)),
    }

    let result = if path.is_file() {
        fs::remove_file(path)
    } else if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        return Err("无效的路径类型".to_string());
    };

    match result {
        Ok(_) => Ok(()),
        Err(e) => match e.raw_os_error() {
            Some(5) => Err("权限不足，请以管理员身份运行程序或检查路径权限".to_string()),
            Some(32) => Err("文件或目录正在被其他程序使用".to_string()),
            Some(2) => Err("文件或目录不存在".to_string()),
            Some(145) => Err("目录不为空".to_string()),
            _ => Err(format!("删除失败: {}", e)),
        },
    }
}

struct SetupState {
    frontend_task: bool,
    backend_task: bool,
}

struct ScanState {
    abort: Arc<AtomicBool>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(SetupState {
            frontend_task: false,
            backend_task: false,
        }))
        .manage(ScanState {
            abort: Arc::new(AtomicBool::new(false)),
        })
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_list_directory,
            calculate_dir_size_simple_fast,
            delete_file,
            set_complete,
            cancel_scan
        ])
        .setup(|app| {
            spawn(setup(app.handle().clone()));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn set_complete(
    app: AppHandle,
    state: State<'_, Mutex<SetupState>>,
    task: String,
) -> Result<(), ()> {
    let mut state_lock = state.lock().unwrap();
    match task.as_str() {
        "frontend" => state_lock.frontend_task = true,
        "backend" => state_lock.backend_task = true,
        _ => panic!("invalid task completed!"),
    }
    if state_lock.backend_task && state_lock.frontend_task {
        let _ = app.get_webview_window("splashscreen").map(|w| w.close());
        if let Some(main) = app.get_webview_window("main") {
            let _ = main.show();
        }
    }
    Ok(())
}

async fn setup(app: AppHandle) -> Result<(), ()> {
    println!("Performing really heavy backend setup task...");
    sleep(Duration::from_secs(1)).await;
    println!("Backend setup task completed!");
    set_complete(
        app.clone(),
        app.state::<Mutex<SetupState>>(),
        "backend".to_string(),
    )
    .await?;
    Ok(())
}