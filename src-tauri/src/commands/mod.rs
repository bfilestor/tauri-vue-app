pub mod ai;
pub mod config;
pub mod file;
pub mod indicator;
pub mod ocr;
pub mod project;
pub mod record;
pub mod system;
pub mod trend;
pub mod mobile;


use std::path::PathBuf;

/// 应用数据目录，注入到 Tauri 状态中
pub struct AppDir(pub PathBuf);
