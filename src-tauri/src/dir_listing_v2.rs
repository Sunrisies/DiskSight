use crate::emit_progress;

use super::models::{Cli, FileEntry};
use super::utils::{human_readable_size, progress_bar_init};
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use tauri::AppHandle;

pub fn list_directory_with_events(
    path: &Path,
    args: &Cli,
    app_handle: &AppHandle,
    abort: Arc<AtomicBool>,
    show_hidden: bool,
) -> Result<Vec<FileEntry>, Error> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("ls: cannot access '{}': {}", path.display(), e);
            return Err(e);
        }
    };

    let mut files: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        if abort.load(Ordering::SeqCst) {
            return Err(Error::new(ErrorKind::Other, "扫描已取消"));
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !show_hidden && file_name.starts_with('.') {
            continue;
        }
        files.push(file_name);
    }

    let total_files = files.len();
    let mut entries = Vec::new();

    if args.long_format {
        let process_pb = progress_bar_init(None).unwrap();
        process_pb.set_message("处理中...");

        for (index, file) in files.iter().enumerate() {
            if abort.load(Ordering::SeqCst) {
                process_pb.finish_and_clear();
                emit_progress(app_handle, path, path, "cancelled");
                return Err(Error::new(
                    ErrorKind::Other,
                    "扫描已取消",
                ));
            }

            emit_progress(app_handle, path, Path::new(file), "processing");
            process_pb.tick();
            let file_path = path.join(file);

            if index % std::cmp::max(1, total_files / 10) == 0 {
                emit_progress(
                    app_handle,
                    path,
                    Path::new(file),
                    &format!("progress_{}%", (index * 100) / total_files),
                );
            }

            if args.name.is_some() {
                let metadata = match file_path.metadata() {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("ls: cannot access '{}': {}", file_path.display(), e);
                        continue;
                    }
                };
                if metadata.is_dir() {
                    if let Some(name) = &args.name {
                        if !file.contains(name) {
                            calculate_dir_size_with_events(
                                file_path,
                                args.human_readable,
                                &process_pb,
                                args.parallel,
                                name,
                                &mut entries,
                                app_handle,
                                abort.clone(),
                            );
                            continue;
                        }
                    }
                } else {
                    continue;
                }
            }

            let metadata = match file_path.metadata() {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("ls: cannot access '{}': {}", file_path.display(), e);
                    continue;
                }
            };

            let (size_display, size_raw) = if metadata.is_dir() {
                emit_progress(app_handle, path, &file_path, "calculating_directory_size");

                let (raw, converted) = calculate_dir_size_with_events_simple(
                    &file_path,
                    args.human_readable,
                    &process_pb,
                    args.parallel,
                    app_handle,
                    abort.clone(),
                );

                if abort.load(Ordering::SeqCst) {
                    process_pb.finish_and_clear();
                    return Err(Error::new(
                        ErrorKind::Other,
                        "扫描已取消",
                    ));
                }

                emit_progress(app_handle, path, &file_path, "directory_calculation_completed");
                (converted, raw)
            } else if args.human_readable {
                (human_readable_size(metadata.len()), metadata.len())
            } else {
                (metadata.len().to_string(), metadata.len())
            };

            entries.push(FileEntry {
                file_type: if metadata.is_dir() { 'd' } else { '-' },
                permissions: format!(
                    "{}-{}-{}",
                    if metadata.permissions().readonly() {
                        "r"
                    } else {
                        " "
                    },
                    "w",
                    "x"
                ),
                size_display,
                size_raw,
                path: match file_path.canonicalize() {
                    Ok(canonical_path) => {
                        let path_str = canonical_path.to_string_lossy().into_owned();
                        let path_str = path_str.strip_prefix(r"\\?\").unwrap_or(&path_str);
                        path_str.to_string()
                    }
                    Err(_e) => file_path.to_string_lossy().into_owned(),
                },
                name: file.to_string(),
                created_time: metadata.created().unwrap_or(SystemTime::UNIX_EPOCH),
            });

            emit_progress(app_handle, path, &file_path, "completed");
        }

        process_pb.finish_and_clear();

        if args.sort {
            entries.sort_by(|a, b| b.size_raw.cmp(&a.size_raw));
        }
    } else {
        for file in files {
            println!("{}", file);
        }
    }

    Ok(entries)
}

fn calculate_dir_size_with_events(
    file_path: PathBuf,
    human_readable: bool,
    pb: &ProgressBar,
    parallel: bool,
    name: &str,
    entries: &mut Vec<FileEntry>,
    app_handle: &AppHandle,
    abort: Arc<AtomicBool>,
) {
    if abort.load(Ordering::SeqCst) {
        return;
    }

    let sub_path_str = file_path.display().to_string();
    let sub_path = Path::new(&sub_path_str);

    emit_progress(app_handle, sub_path, sub_path, "searching_in_directory");

    let sub_entries = match fs::read_dir(sub_path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("ls: cannot access '{}': {}", sub_path.display(), e);
            return;
        }
    };

    for entry in sub_entries.flatten() {
        if abort.load(Ordering::SeqCst) {
            return;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("ls: cannot access '{}': {}", sub_path.display(), e);
                continue;
            }
        };

        emit_progress(app_handle, sub_path, &entry.path(), "checking_file");

        if metadata.is_dir() {
            let file_path = sub_path.join(&file_name);
            if !file_name.contains(name) {
                calculate_dir_size_with_events(
                    file_path,
                    human_readable,
                    pb,
                    parallel,
                    name,
                    entries,
                    app_handle,
                    abort.clone(),
                );
                continue;
            } else {
                emit_progress(app_handle, sub_path, &file_path, "calculating_matching_directory");

                let (raw, converted) = calculate_dir_size_with_events_simple(
                    &file_path,
                    human_readable,
                    pb,
                    parallel,
                    app_handle,
                    abort.clone(),
                );

                if abort.load(Ordering::SeqCst) {
                    return;
                }

                entries.push(FileEntry {
                    file_type: if metadata.is_dir() { 'd' } else { '-' },
                    permissions: format!(
                        "{}-{}-{}",
                        if metadata.permissions().readonly() {
                            "r"
                        } else {
                            " "
                        },
                        "w",
                        "x"
                    ),
                    size_display: converted,
                    size_raw: raw,
                    path: match file_path.canonicalize() {
                        Ok(canonical_path) => {
                            let path_str = canonical_path.to_string_lossy().into_owned();
                            let path_str = path_str.strip_prefix(r"\\?\").unwrap_or(&path_str);
                            path_str.to_string()
                        }
                        Err(e) => {
                            eprintln!("获取绝对路径失败: {}", e);
                            "".to_string()
                        }
                    },
                    name: file_name,
                    created_time: metadata.created().unwrap_or(SystemTime::UNIX_EPOCH),
                });

                emit_progress(app_handle, sub_path, &file_path, "matching_directory_completed");
            }
        }
    }
}

pub fn calculate_dir_size_with_events_simple(
    path: &Path,
    human_readable: bool,
    main_pb: &ProgressBar,
    parallel: bool,
    app_handle: &AppHandle,
    abort: Arc<AtomicBool>,
) -> (u64, String) {
    fn inner_calculate(
        p: &Path,
        pb: &ProgressBar,
        parallel: bool,
        app_handle: &AppHandle,
        abort: Arc<AtomicBool>,
    ) -> u64 {
        if abort.load(Ordering::SeqCst) {
            return 0;
        }
        match fs::read_dir(p) {
            Ok(entries) => {
                let mut total_size = 0;
                let entries: Vec<_> = entries
                    .filter_map(|e| {
                        pb.tick();
                        if abort.load(Ordering::SeqCst) {
                            return None;
                        }
                        match e {
                            Ok(entry) => {
                                emit_progress(app_handle, p, &entry.path(), "processing_file");
                                Some(entry)
                            }
                            Err(e) => {
                                eprintln!("无法读取目录项 {}: {}", p.display(), e);
                                None
                            }
                        }
                    })
                    .collect();

                if parallel {
                    total_size += entries
                        .par_iter()
                        .map(|e| process_entry_with_events(e, pb, parallel, app_handle, abort.clone()))
                        .sum::<u64>();
                } else {
                    total_size += entries
                        .iter()
                        .map(|e| process_entry_with_events(e, pb, parallel, app_handle, abort.clone()))
                        .sum::<u64>();
                }

                total_size
            }
            Err(e) => {
                eprintln!("无法读取目录 {}: {}", p.display(), e);
                0
            }
        }
    }

    fn process_entry_with_events(
        e: &std::fs::DirEntry,
        pb: &ProgressBar,
        parallel: bool,
        app_handle: &AppHandle,
        abort: Arc<AtomicBool>,
    ) -> u64 {
        if abort.load(Ordering::SeqCst) {
            return 0;
        }
        match e.metadata() {
            Ok(metadata) => {
                if metadata.is_dir() {
                    inner_calculate(&e.path(), pb, parallel, app_handle, abort)
                } else {
                    metadata.len()
                }
            }
            Err(e) => {
                eprintln!("无法获取文件元数据 {}", e);
                0
            }
        }
    }

    main_pb.set_message(format!("计算 {}...", path.display()));
    emit_progress(app_handle, path, path, "calculating_directory_size");

    let total = inner_calculate(path, main_pb, parallel, app_handle, abort.clone());
    if abort.load(Ordering::SeqCst) {
        return (0, String::from("已取消"));
    }
    main_pb.set_message("处理中...");

    let converted = if human_readable {
        human_readable_size(total)
    } else {
        total.to_string()
    };
    (total, converted)
}