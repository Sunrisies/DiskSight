use disk_sight::{human_readable_size, list_directory, Cli, FileEntry};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        // initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "DiskSight - 目录文件大小查看器",
        options,
        Box::new(|_cc| Ok(Box::new(FileSizeViewer::default()))),
    )
}

struct FileSizeViewer {
    current_path: String,
    entries: Arc<Mutex<Vec<FileEntry>>>,
    total_count: usize,
    total_size: u64,
    dark_mode: bool,
    last_refresh: std::time::Instant,
    is_loading: Arc<AtomicBool>, // 添加加载状态
}

impl Default for FileSizeViewer {
    fn default() -> Self {
        let current_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .display()
            .to_string();

        let entries = Arc::new(Mutex::new(Vec::new()));
        let is_loading = Arc::new(AtomicBool::new(true)); // 初始化为加载状态

        let mut viewer = Self {
            current_path: current_dir,
            entries,
            total_count: 0,
            total_size: 0,
            dark_mode: false,
            last_refresh: std::time::Instant::now(),
            is_loading,
        };

        viewer.refresh_data();
        viewer
    }
}

impl FileSizeViewer {
    fn refresh_data(&mut self) {
        let path = self.current_path.clone();
        println!("Refreshing data for path: {}", path);
        let entries = Arc::clone(&self.entries);
        let is_loading = Arc::clone(&self.is_loading);

        // 设置加载状态为true
        is_loading.store(true, Ordering::SeqCst);

        thread::spawn(move || {
            let arg = Cli {
                // path: std::env::current_dir().unwrap(),
                file: None,
                long_format: true,
                human_readable: true,
                all: true,
                name: None,
                show_time: true,
                parallel: true,
                sort: true,
                full_path: true,
            };
            match list_directory(Path::new(&path), &arg) {
                Ok(local_entries) => {
                    let mut entries_lock = entries.lock().unwrap();
                    *entries_lock = local_entries;
                }
                Err(e) => {
                    eprintln!("Error listing directory: {}", e);
                    let mut entries_lock = entries.lock().unwrap();
                    *entries_lock = Vec::new(); // 出错时设置为空向量
                }
            }
            // 数据加载完成，设置加载状态为false
            is_loading.store(false, Ordering::SeqCst);
        });

        self.last_refresh = std::time::Instant::now();
    }

    fn select_directory(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.current_path = path.display().to_string();
            self.refresh_data();
        }
    }
}

impl eframe::App for FileSizeViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 设置中文字体
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "SimHei".to_owned(),
            egui::FontData::from_static(include_bytes!("../fonts/SimHei.ttf")).into(), // 需要提供中文字体文件
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "SimHei".to_owned());
        ctx.set_fonts(fonts);

        // 设置暗黑/浅色模式
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("目录文件大小查看器");

            // 路径选择和刷新控制
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label("当前目录:");
                ui.text_edit_singleline(&mut self.current_path);

                // 在加载时禁用按钮
                let is_loading = self.is_loading.load(Ordering::SeqCst);

                // 使用 egui 推荐的方式设置控件启用状态
                let response = ui.add_enabled(!is_loading, egui::Button::new("浏览..."));
                if response.clicked() {
                    self.select_directory();
                }

                let response = ui.add_enabled(!is_loading, egui::Button::new("刷新"));
                if response.clicked() {
                    self.refresh_data();
                }
            });

            ui.separator();

            // 显示文件/目录表格
            let entries = self.entries.lock().unwrap();
            self.total_count = entries.len();
            self.total_size = entries.iter().map(|e| e.size_raw).sum();

            // 检查是否正在加载
            let is_loading = self.is_loading.load(Ordering::SeqCst);

            if is_loading {
                // 显示加载指示器
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.spinner();
                    ui.add_space(10.0);
                    ui.label("正在加载目录内容...");
                });
            } else if entries.is_empty() {
                // 显示空目录消息
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.label("目录为空或无法访问");
                });
            } else {
                // 创建表格
                egui::ScrollArea::both().show(ui, |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::auto().at_least(40.0)) // 类型列
                        .column(Column::auto().at_least(40.0)) // 权限列
                        .column(Column::auto().at_least(80.0)) // 大小列
                        .column(Column::remainder()) // 路径列
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.heading("类型");
                            });
                            header.col(|ui| {
                                ui.heading("权限");
                            });
                            header.col(|ui| {
                                ui.heading("大小");
                            });
                            header.col(|ui| {
                                ui.heading("路径");
                            });
                        })
                        .body(|mut body| {
                            for entry in entries.iter() {
                                body.row(20.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(entry.file_type.to_string());
                                    });
                                    row.col(|ui| {
                                        ui.label(&entry.permissions);
                                    });
                                    row.col(|ui| {
                                        ui.label(&entry.size_display);
                                    });
                                    row.col(|ui| {
                                        ui.label(&entry.name);
                                    });
                                });
                            }
                        });
                });
            }

            ui.separator();

            // 显示统计信息
            ui.horizontal(|ui| {
                ui.label(format!(
                    "总数量: {} | 总大小: {}",
                    self.total_count,
                    human_readable_size(self.total_size)
                ));
            });

            // 主题切换
            ui.horizontal(|ui| {
                ui.label("主题:");
                if ui
                    .button(if self.dark_mode {
                        "浅色模式"
                    } else {
                        "暗黑模式"
                    })
                    .clicked()
                {
                    self.dark_mode = !self.dark_mode;
                }
            });
        });
    }
}
