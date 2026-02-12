pub mod config;
pub mod project;
pub mod indicator;
pub mod record;
pub mod file;
pub mod ocr;
pub mod ai;
pub mod trend;
pub mod system;

use std::path::PathBuf;

/// 应用数据目录，注入到 Tauri 状态中
pub struct AppDir(pub PathBuf);
