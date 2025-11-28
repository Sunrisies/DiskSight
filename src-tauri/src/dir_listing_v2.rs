use crate::emit_progress;

use super::models::{Cli, FileEntry};
use super::utils::{human_readable_size, progress_bar_init};
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

// 使用事件系统的目录列表函数
pub fn list_directory_with_events(
    path: &Path,
    args: &Cli,
    app_handle: &AppHandle,
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
        let file_name = entry.file_name().to_string_lossy().to_string();
        files.push(file_name);
    }
    let sorted_files = files.clone();
    files.sort();
    let total_files = sorted_files.len();
    let mut entries = Vec::new();

    if args.long_format {
        let process_pb = progress_bar_init(None).unwrap();
        process_pb.set_message("处理中...");

        for (index, file) in sorted_files.iter().enumerate() {
            // 发送处理进度事件
            emit_progress(app_handle, path, Path::new(file), "processing");

            process_pb.tick();
            let file_path = path.join(file);

            // 只在处理大文件或每10%进度时报告
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
                // 发送开始计算目录大小事件
                emit_progress(app_handle, path, &file_path, "calculating_directory_size");

                let (raw, converted) = calculate_dir_size_with_events_simple(
                    &file_path,
                    args.human_readable,
                    &process_pb,
                    args.parallel,
                    app_handle,
                );

                // 发送完成目录计算事件
                emit_progress(
                    app_handle,
                    path,
                    &file_path,
                    "directory_calculation_completed",
                );
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
                created_time: metadata.created()?,
            });

            // 发送完成当前文件事件
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

// 使用事件系统的目录搜索函数
fn calculate_dir_size_with_events(
    file_path: PathBuf,
    human_readable: bool,
    pb: &ProgressBar,
    parallel: bool,
    name: &str,
    entries: &mut Vec<FileEntry>,
    app_handle: &AppHandle,
) {
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
                );
                continue;
            } else {
                emit_progress(
                    app_handle,
                    sub_path,
                    &file_path,
                    "calculating_matching_directory",
                );

                let (raw, converted) = calculate_dir_size_with_events_simple(
                    &file_path,
                    human_readable,
                    pb,
                    parallel,
                    app_handle,
                );

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
                    created_time: metadata.created().unwrap_or(std::time::SystemTime::now()),
                });

                emit_progress(
                    app_handle,
                    sub_path,
                    &file_path,
                    "matching_directory_completed",
                );
            }
        }
    }
}

// 使用事件系统的目录大小计算函数
pub fn calculate_dir_size_with_events_simple(
    path: &Path,
    human_readable: bool,
    main_pb: &ProgressBar,
    parallel: bool,
    app_handle: &AppHandle,
) -> (u64, String) {
    fn inner_calculate(p: &Path, pb: &ProgressBar, parallel: bool, app_handle: &AppHandle) -> u64 {
        match fs::read_dir(p) {
            Ok(entries) => {
                let mut total_size = 0;
                let entries: Vec<_> = entries
                    .filter_map(|e| {
                        pb.tick();
                        match e {
                            Ok(entry) => {
                                // 发送处理文件事件
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
                        .map(|e| process_entry_with_events(e, pb, parallel, app_handle))
                        .sum::<u64>();
                } else {
                    total_size += entries
                        .iter()
                        .map(|e| process_entry_with_events(e, pb, parallel, app_handle))
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
    ) -> u64 {
        match e.metadata() {
            Ok(metadata) => {
                if metadata.is_dir() {
                    inner_calculate(&e.path(), pb, parallel, app_handle)
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

    let total = inner_calculate(path, main_pb, parallel, app_handle);
    main_pb.set_message("处理中...");

    let converted = if human_readable {
        human_readable_size(total)
    } else {
        total.to_string()
    };
    (total, converted)
}
