use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Cli {
    pub file: Option<String>,
    pub long_format: bool,
    pub human_readable: bool,
    pub all: bool,
    pub show_time: bool,
    pub parallel: bool,
    pub sort: bool,
    pub name: Option<String>,
    pub full_path: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileEntry {
    /// 文件类型
    pub file_type: char,
    /// 文件权限
    pub permissions: String,
    /// 文件原始显示大小
    pub size_raw: u64,
    /// 文件大小显示
    pub size_display: String,
    /// 文件创建时间
    pub created_time: SystemTime,
    pub path: String,
    /// 文件名
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct DirectoryResult {
    pub entries: Vec<FileEntry>,
    pub query_time: f64,
}

#[derive(Clone, Serialize)]
pub struct ProgressEvent {
    pub current_path: String,
    pub current_file: String,
    pub status: String,
}
