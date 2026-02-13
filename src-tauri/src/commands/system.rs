use tauri::State;
use crate::db::Database;
use super::AppDir;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use std::io::{Read, Write};

/// 重置检查数据 (删除历史记录、图片、OCR、AI分析等)
#[tauri::command]
pub async fn reset_checkup_data(db: State<'_, Database>, app_dir: State<'_, AppDir>) -> Result<(), String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    
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
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    conn.execute("DELETE FROM indicators", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM checkup_projects", []).map_err(|e| e.to_string())?;

    Ok(())
}

/// 备份数据
#[tauri::command]
pub async fn backup_data(target_path: String, app_dir: State<'_, AppDir>, db: State<'_, Database>) -> Result<(), String> {
    // 1. 关闭数据库以确保文件一致性
    db.close()?;

    let result = (|| -> Result<(), String> {
        // 2 Create zip file
        let file = fs::File::create(&target_path).map_err(|e| format!("创建备份文件失败: {}", e))?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        // 3. Add Database
        let db_path = app_dir.0.join("health_guard.db");
        if db_path.exists() {
            zip.start_file("health_guard.db", options).map_err(|e| e.to_string())?;
            let mut f = fs::File::open(&db_path).map_err(|e| e.to_string())?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
            zip.write_all(&buffer).map_err(|e| e.to_string())?;
        }

        // 4. Add Pictures
        let pictures_dir = app_dir.0.join("pictures");
        if pictures_dir.exists() {
            for entry in WalkDir::new(&pictures_dir) {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if path.is_file() {
                    let name = path.strip_prefix(&app_dir.0).map_err(|e| e.to_string())?;
                    // Ensure name has forward slashes
                    let name_str = name.to_string_lossy().replace("\\", "/");
                    
                    zip.start_file(name_str, options).map_err(|e| e.to_string())?;
                    let mut f = fs::File::open(path).map_err(|e| e.to_string())?;
                    let mut buffer = Vec::new();
                    f.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
                    zip.write_all(&buffer).map_err(|e| e.to_string())?;
                }
            }
        }
        
        zip.finish().map_err(|e| format!("写入备份失败: {}", e))?;
        Ok(())
    })();

    // 5. Reopen database strictly
    db.reopen()?;
    
    result
}

/// 恢复数据
#[tauri::command]
pub async fn restore_data(source_path: String, app_dir: State<'_, AppDir>, db: State<'_, Database>) -> Result<(), String> {
    // 1. Close DB
    db.close()?;

    // 定义内部闭包以便在出错时也能尝试 reopen
    let result = (|| -> Result<(), String> {
        // 2. Open Zip
        let file = fs::File::open(&source_path).map_err(|e| format!("打开备份文件失败: {}", e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("解析压缩包失败: {}", e))?;

        // 3. Clear existing pictures?
        // Maybe we should clear pictures dir before restore to avoid stale files? 
        // User asked "extract restore", usually implies sync. Let's just overwrite.
        
        // 4. Extract
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = match file.enclosed_name() {
                Some(path) => app_dir.0.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).map_err(|e| e.to_string())?;
                    }
                }
                let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
            }
        }
        
        // 5. Cleanup WAL/SHM files
        let wal_path = app_dir.0.join("health_guard.db-wal");
        let shm_path = app_dir.0.join("health_guard.db-shm");
        if wal_path.exists() { fs::remove_file(wal_path).ok(); }
        if shm_path.exists() { fs::remove_file(shm_path).ok(); }

        Ok(())
    })();

    // 6. Reopen DB
    db.reopen()?;

    result
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
