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
use tauri::async_runtime::spawn;
use tauri::async_runtime::spawn_blocking;
use tauri::Emitter;
use tauri::{AppHandle, Manager, State};

use tokio::time::{sleep, Duration};
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
#[tauri::command]
async fn delete_file(path: String, force: bool) -> Result<(), String> {
    let path = Path::new(&path);

    // 检查路径是否存在
    if !path.exists() {
        return Err("路径不存在".to_string());
    }

    // 检查路径是否可写
    match fs::metadata(&path) {
        Ok(metadata) => {
            if metadata.permissions().readonly() {
                if !force {
                    return Err("路径是只读的。如要强制删除，请设置 force 参数为 true".to_string());
                }
                // 尝试移除只读属性
                let mut perms = metadata.permissions();
                perms.set_readonly(false);
                if let Err(e) = fs::set_permissions(path, perms) {
                    return Err(format!("无法修改路径权限: {}", e));
                }
            }
        }
        Err(e) => return Err(format!("无法访问路径: {}", e)),
    }

    // 根据路径类型选择删除方法
    let result = if path.is_file() {
        fs::remove_file(path)
    } else if path.is_dir() {
        // 对于目录，需要递归删除
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
// 创建一个结构，用于跟踪前端任务完成情况
// 设置相关任务
struct SetupState {
    frontend_task: bool,
    backend_task: bool,
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(SetupState {
            frontend_task: false,
            backend_task: false,
        }))
        // 添加我们用于检查的命令
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_list_directory,
            calculate_dir_size_simple_fast,
            delete_file,
            set_complete
        ])
        .setup(|app| {
            // Spawn 操作设置为一个非阻塞任务，以便在它执行的同时可以创建并运行窗口。
            spawn(setup(app.handle().clone()));
            // 钩子期望返回一个 Ok 的结果
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// 一个用于设置初始化任务状态的自定义任务
#[tauri::command]
async fn set_complete(
    app: AppHandle,
    state: State<'_, Mutex<SetupState>>,
    task: String,
) -> Result<(), ()> {
    // 以只读方式锁定 `State`
    let mut state_lock = state.lock().unwrap();
    match task.as_str() {
        "frontend" => state_lock.frontend_task = true,
        "backend" => state_lock.backend_task = true,
        _ => panic!("invalid task completed!"),
    }
    // 检查两个任务是否都已完成
    if state_lock.backend_task && state_lock.frontend_task {
        // 设置都已完成，我们可以关闭启动画面并且显示 main 窗口了
        let splash_window = app.get_webview_window("splashscreen").unwrap();
        let main_window = app.get_webview_window("main").unwrap();
        splash_window.close().unwrap();
        main_window.show().unwrap();
    }
    Ok(())
}

// 一个异步函数，用于执行一些耗时的设置任务
async fn setup(app: AppHandle) -> Result<(), ()> {
    // 模拟执行一些耗时的设置任务，3秒后完成
    println!("Performing really heavy backend setup task...");
    sleep(Duration::from_secs(1)).await;
    println!("Backend setup task completed!");
    // 设置后端任务为已完成
    // 可以像普通函数一样运行命令，但需要自己处理输入参数
    set_complete(
        app.clone(),
        app.state::<Mutex<SetupState>>(),
        "backend".to_string(),
    )
    .await?;
    Ok(())
}
