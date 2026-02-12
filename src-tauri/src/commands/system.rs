use tauri::State;
use crate::db::Database;
use super::AppDir;
use std::fs;
use std::path::Path;

/// 重置检查数据 (删除历史记录、图片、OCR、AI分析等)
#[tauri::command]
pub async fn reset_checkup_data(db: State<'_, Database>, app_dir: State<'_, AppDir>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    // 1. 删除数据库中的相关记录
    // 注意：因无级联删除，需按顺序手动删除
    conn.execute("DELETE FROM indicator_values", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM ocr_results", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM ai_analyses", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM checkup_files", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM checkup_records", []).map_err(|e| e.to_string())?;
    
    // 2. 删除图片目录下的所有内容 (仅 pictures 目录)
    let pictures_dir = app_dir.0.join("pictures");
    if pictures_dir.exists() {
        if let Err(e) = remove_dir_contents(&pictures_dir) {
             return Err(format!("删除图片失败: {}", e));
        }
    }
    
    Ok(())
}

/// 重置全部数据 (包括检查项目设置)
#[tauri::command]
pub async fn reset_all_data(db: State<'_, Database>, app_dir: State<'_, AppDir>) -> Result<(), String> {
    // 1. 重置检查数据
    reset_checkup_data(db.clone(), app_dir.clone()).await?;

    // 2. 删除项目和指标设置
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM indicators", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM checkup_projects", []).map_err(|e| e.to_string())?;

    Ok(())
}

/// 删除目录内容但不删除目录本身
fn remove_dir_contents(dir: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}
