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

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub file_type: char,
    pub permissions: String,
    pub size_raw: u64,
    pub size_display: String,
    pub path: String,
    pub name: String,
}
