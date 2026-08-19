use super::models::{Cli, FileEntry, ProgressEvent};
use super::utils::human_readable_size;
use rayon::prelude::*;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::SystemTime;
use tauri::AppHandle;
use tauri::Emitter;

pub fn calculate_dir_size(
    path: &Path,
    human_readable: bool,
    parallel: bool,
    abort: &Arc<AtomicBool>,
) -> (u64, String) {
    fn inner_calculate(p: &Path, parallel: bool, abort: &Arc<AtomicBool>) -> u64 {
        if abort.load(Ordering::Relaxed) {
            return 0;
        }
        match fs::read_dir(p) {
            Ok(entries) => {
                let items: Vec<_> = entries
                    .filter_map(|e| {
                        if abort.load(Ordering::Relaxed) {
                            return None;
                        }
                        e.ok()
                    })
                    .collect();

                if parallel {
                    items
                        .par_iter()
                        .map(|e| process_entry_size(e, parallel, abort))
                        .sum::<u64>()
                } else {
                    items
                        .iter()
                        .map(|e| process_entry_size(e, parallel, abort))
                        .sum::<u64>()
                }
            }
            Err(e) => {
                eprintln!("无法读取目录 {}: {}", p.display(), e);
                0
            }
        }
    }

    fn process_entry_size(
        e: &std::fs::DirEntry,
        parallel: bool,
        abort: &Arc<AtomicBool>,
    ) -> u64 {
        if abort.load(Ordering::Relaxed) {
            return 0;
        }
        match e.metadata() {
            Ok(metadata) => {
                if metadata.is_dir() {
                    inner_calculate(&e.path(), parallel, abort)
                } else {
                    metadata.len()
                }
            }
            Err(_) => 0,
        }
    }

    let total = inner_calculate(path, parallel, abort);
    if abort.load(Ordering::Relaxed) {
        return (0, String::from("已取消"));
    }

    let converted = if human_readable {
        human_readable_size(total)
    } else {
        total.to_string()
    };
    (total, converted)
}

pub fn list_directory(
    path: &Path,
    args: &Cli,
    abort: &Arc<AtomicBool>,
    show_hidden: bool,
) -> Result<Vec<FileEntry>, Error> {
    list_directory_core(path, args, abort, show_hidden, &None, &None)
}

pub fn list_directory_with_events(
    path: &Path,
    args: &Cli,
    app_handle: &AppHandle,
    abort: &Arc<AtomicBool>,
    show_hidden: bool,
) -> Result<Vec<FileEntry>, Error> {
    let emit_counter = Arc::new(AtomicUsize::new(0));
    let app_handle_opt = Some(app_handle.clone());
    list_directory_core(path, args, abort, show_hidden, &app_handle_opt, &Some(emit_counter))
}

fn list_directory_core(
    path: &Path,
    args: &Cli,
    abort: &Arc<AtomicBool>,
    show_hidden: bool,
    app_handle: &Option<AppHandle>,
    emit_counter: &Option<Arc<AtomicUsize>>,
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
        if abort.load(Ordering::Relaxed) {
            return Err(Error::new(ErrorKind::Other, "扫描已取消"));
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !show_hidden && file_name.starts_with('.') {
            continue;
        }
        files.push(file_name);
    }

    files.sort();
    let total_files = files.len();

    if args.name.is_some() {
        return list_directory_search(path, args, abort, show_hidden, app_handle, emit_counter, &files);
    }

    if args.long_format {
        if args.parallel {
            return list_directory_parallel(path, args, abort, app_handle, emit_counter, &files, total_files);
        }

        let mut entries_result = Vec::with_capacity(files.len());

        for (index, file) in files.iter().enumerate() {
            if abort.load(Ordering::Relaxed) {
                return Err(Error::new(ErrorKind::Other, "扫描已取消"));
            }

            let file_path = path.join(file);
            let metadata = match file_path.metadata() {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("ls: cannot access '{}': {}", file_path.display(), e);
                    continue;
                }
            };

            let (size_display, size_raw) = if metadata.is_dir() {
                emit_batch(app_handle, emit_counter, path, &file_path, "calculating_directory_size", abort);
                let (raw, converted) =
                    calculate_dir_size(&file_path, args.human_readable, args.parallel, abort);
                if abort.load(Ordering::Relaxed) {
                    return Err(Error::new(ErrorKind::Other, "扫描已取消"));
                }
                emit_batch(app_handle, emit_counter, path, &file_path, "directory_calculation_completed", abort);
                (converted, raw)
            } else if args.human_readable {
                (human_readable_size(metadata.len()), metadata.len())
            } else {
                (metadata.len().to_string(), metadata.len())
            };

            entries_result.push(build_file_entry(&file_path, &metadata, file, path, size_display, size_raw));

            if index % 10 == 0 {
                emit_batch(app_handle, emit_counter, path, &file_path, "processing", abort);
            }
        }

        if args.sort {
            entries_result.sort_by(|a, b| b.size_raw.cmp(&a.size_raw));
        }

        return Ok(entries_result);
    }

    Ok(Vec::new())
}

fn list_directory_parallel(
    path: &Path,
    args: &Cli,
    abort: &Arc<AtomicBool>,
    app_handle: &Option<AppHandle>,
    emit_counter: &Option<Arc<AtomicUsize>>,
    files: &[String],
    _total_files: usize,
) -> Result<Vec<FileEntry>, Error> {
    let path = Arc::new(path.to_path_buf());

    let results: Vec<Result<FileEntry, Error>> = files
        .par_iter()
        .enumerate()
        .filter_map(|(index, file)| {
            if abort.load(Ordering::Relaxed) {
                return Some(Err(Error::new(ErrorKind::Other, "扫描已取消")));
            }

if index % 10 == 0 {
            emit_batch(app_handle, emit_counter, &path, &path.join(file), "processing", abort);
        }

            let file_path = path.join(file);
            let metadata = match file_path.metadata() {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("ls: cannot access '{}': {}", file_path.display(), e);
                    return None;
                }
            };

            if !metadata.is_dir() {
                let size_display = if args.human_readable {
                    human_readable_size(metadata.len())
                } else {
                    metadata.len().to_string()
                };
                return Some(Ok(build_file_entry(&file_path, &metadata, file, &path, size_display, metadata.len())));
            }

            emit_batch(app_handle, emit_counter, &path, &file_path, "calculating_directory_size", abort);
            let (raw, converted) =
                calculate_dir_size(&file_path, args.human_readable, args.parallel, abort);

            if abort.load(Ordering::Relaxed) {
                return Some(Err(Error::new(ErrorKind::Other, "扫描已取消")));
            }

            emit_batch(app_handle, emit_counter, &path, &file_path, "directory_calculation_completed", abort);
            Some(Ok(build_file_entry(&file_path, &metadata, file, &path, converted, raw)))
        })
        .collect();

    let mut entries: Vec<FileEntry> = Vec::with_capacity(results.len());
    for result in results {
        match result {
            Ok(entry) => entries.push(entry),
            Err(e) => {
                if e.to_string().contains("扫描已取消") {
                    return Err(Error::new(ErrorKind::Other, "扫描已取消"));
                }
            }
        }
    }

    if args.sort {
        entries.sort_by(|a, b| b.size_raw.cmp(&a.size_raw));
    }
    Ok(entries)
}

fn list_directory_search(
    path: &Path,
    args: &Cli,
    abort: &Arc<AtomicBool>,
    _show_hidden: bool,
    app_handle: &Option<AppHandle>,
    emit_counter: &Option<Arc<AtomicUsize>>,
    files: &[String],
) -> Result<Vec<FileEntry>, Error> {
    let mut entries_result = Vec::new();

    for (index, file) in files.iter().enumerate() {
        if abort.load(Ordering::Relaxed) {
            return Err(Error::new(ErrorKind::Other, "扫描已取消"));
        }

        let file_path = path.join(file);
        let metadata = match file_path.metadata() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("ls: cannot access '{}': {}", file_path.display(), e);
                continue;
            }
        };

        if !metadata.is_dir() {
            continue;
        }

        if let Some(name) = &args.name {
            if file.contains(name) {
                emit_batch(app_handle, emit_counter, path, &file_path, "calculating_directory_size", abort);
                let (raw, converted) =
                    calculate_dir_size(&file_path, args.human_readable, args.parallel, abort);
                if abort.load(Ordering::Relaxed) {
                    return Err(Error::new(ErrorKind::Other, "扫描已取消"));
                }
                emit_batch(app_handle, emit_counter, path, &file_path, "directory_calculation_completed", abort);
                entries_result.push(build_file_entry(&file_path, &metadata, file, path, converted, raw));
            } else {
                search_subdirs(&file_path, name, args.human_readable, args.parallel, abort, app_handle, emit_counter, &mut entries_result);
            }
        }

        if index % 100 == 0 {
            emit_batch(app_handle, emit_counter, path, &file_path, "searching", abort);
        }
    }

    if args.sort {
        entries_result.sort_by(|a, b| b.size_raw.cmp(&a.size_raw));
    }

    Ok(entries_result)
}

fn search_subdirs(
    dir_path: &Path,
    name: &str,
    human_readable: bool,
    parallel: bool,
    abort: &Arc<AtomicBool>,
    app_handle: &Option<AppHandle>,
    emit_counter: &Option<Arc<AtomicUsize>>,
    entries: &mut Vec<FileEntry>,
) {
    if abort.load(Ordering::Relaxed) {
        return;
    }

    let sub_entries = match fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in sub_entries.flatten() {
        if abort.load(Ordering::Relaxed) {
            return;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();
        let sub_path = dir_path.join(&file_name);

        if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            continue;
        }

        if file_name.contains(name) {
            emit_batch(app_handle, emit_counter, dir_path, &sub_path, "calculating_matching_directory", abort);
            let (raw, converted) = calculate_dir_size(&sub_path, human_readable, parallel, abort);
            if abort.load(Ordering::Relaxed) {
                return;
            }
            match sub_path.metadata() {
                Ok(meta) => {
                    entries.push(build_file_entry(&sub_path, &meta, &file_name, dir_path, converted, raw));
                }
                Err(_) => {}
            }
            emit_batch(app_handle, emit_counter, dir_path, &sub_path, "matching_directory_completed", abort);
        } else {
            search_subdirs(&sub_path, name, human_readable, parallel, abort, app_handle, emit_counter, entries);
        }
    }
}

fn build_file_entry(
    _file_path: &Path,
    metadata: &std::fs::Metadata,
    file_name: &str,
    parent_path: &Path,
    size_display: String,
    size_raw: u64,
) -> FileEntry {
    let path_str = parent_path
        .join(file_name)
        .to_string_lossy()
        .to_string();
    let path_str = path_str.strip_prefix(r"\\?\").unwrap_or(&path_str).to_string();

    FileEntry {
        file_type: if metadata.is_dir() { 'd' } else { '-' },
        permissions: format!(
            "{}-{}-{}",
            if metadata.permissions().readonly() { "r" } else { " " },
            "w",
            "x"
        ),
        size_display,
        size_raw,
        path: path_str,
        name: file_name.to_string(),
        created_time: metadata.created().unwrap_or(SystemTime::UNIX_EPOCH),
    }
}

fn emit_batch(
    ah: &Option<AppHandle>,
    _emit_counter: &Option<Arc<AtomicUsize>>,
    current_path: &Path,
    current_file: &Path,
    status: &str,
    abort: &Arc<AtomicBool>,
) {
    if abort.load(Ordering::Relaxed) {
        return;
    }
    if let Some(ah) = ah {
        let _ = ah.emit(
            "scan-progress",
            ProgressEvent {
                current_path: current_path.to_string_lossy().to_string(),
                current_file: current_file.to_string_lossy().to_string(),
                status: status.to_string(),
            },
        );
    }
}
