use serde::{Deserialize, Serialize};
use tauri::State;
use crate::db::Database;
use super::AppDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckupFile {
    pub id: String,
    pub record_id: String,
    pub project_id: String,
    pub project_name: Option<String>,
    pub original_filename: String,
    pub stored_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub uploaded_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UploadFileInput {
    pub record_id: String,
    pub project_id: String,
    pub checkup_date: String,
    /// Base64 编码的文件内容
    pub file_data: String,
    pub filename: String,
}

/// 批量上传文件
#[tauri::command]
pub fn upload_files(
    files: Vec<UploadFileInput>,
    db: State<Database>,
    app_dir: State<AppDir>,
) -> Result<Vec<CheckupFile>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let now = chrono::Local::now().to_rfc3339();
    let mut result = Vec::new();

    for file_input in files {
        let id = uuid::Uuid::new_v4().to_string();

        // 获取项目名称用于目录结构
        let project_name: String = conn
            .query_row(
                "SELECT name FROM checkup_projects WHERE id = ?1",
                [&file_input.project_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("项目不存在: {}", e))?;

        // 构建存储路径: pictures/<项目名>/<日期>/<文件名>
        let store_dir = app_dir.0
            .join("pictures")
            .join(&project_name)
            .join(&file_input.checkup_date);

        std::fs::create_dir_all(&store_dir)
            .map_err(|e| format!("创建目录失败: {}", e))?;

        // 解码 base64 文件数据
        let file_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &file_input.file_data,
        )
        .map_err(|e| format!("文件解码失败: {}", e))?;

        let file_size = file_bytes.len() as i64;

        // 避免文件名冲突：加上UUID前缀
        let stored_filename = format!("{}_{}", &id[..8], &file_input.filename);
        let stored_path = store_dir.join(&stored_filename);

        std::fs::write(&stored_path, &file_bytes)
            .map_err(|e| format!("文件保存失败: {}", e))?;

        // 获取相对路径
        let relative_path = stored_path
            .strip_prefix(&app_dir.0)
            .unwrap_or(&stored_path)
            .to_string_lossy()
            .to_string();

        // 推断 MIME 类型
        let mime_type = guess_mime_type(&file_input.filename);

        // 插入数据库
        conn.execute(
            "INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, file_size, mime_type, uploaded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![id, file_input.record_id, file_input.project_id, file_input.filename, relative_path, file_size, mime_type, now],
        )
        .map_err(|e| format!("保存文件记录失败: {}", e))?;

        result.push(CheckupFile {
            id,
            record_id: file_input.record_id.clone(),
            project_id: file_input.project_id.clone(),
            project_name: Some(project_name),
            original_filename: file_input.filename,
            stored_path: relative_path,
            file_size,
            mime_type,
            uploaded_at: now.clone(),
        });
    }

    // 更新检查记录状态
    if let Some(first) = result.first() {
        conn.execute(
            "UPDATE checkup_records SET status = 'pending_ocr', updated_at = ?1 WHERE id = ?2 AND status = 'pending_upload'",
            rusqlite::params![now, first.record_id],
        ).ok();
    }

    Ok(result)
}

/// 获取某次检查记录的所有文件
#[tauri::command]
pub fn list_files(record_id: String, db: State<Database>) -> Result<Vec<CheckupFile>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT f.id, f.record_id, f.project_id, p.name, f.original_filename, f.stored_path, f.file_size, f.mime_type, f.uploaded_at
             FROM checkup_files f
             LEFT JOIN checkup_projects p ON f.project_id = p.id
             WHERE f.record_id = ?1
             ORDER BY p.name ASC, f.uploaded_at ASC"
        )
        .map_err(|e| format!("查询文件失败: {}", e))?;

    let files = stmt
        .query_map([&record_id], |row| {
            Ok(CheckupFile {
                id: row.get(0)?,
                record_id: row.get(1)?,
                project_id: row.get(2)?,
                project_name: row.get(3)?,
                original_filename: row.get(4)?,
                stored_path: row.get(5)?,
                file_size: row.get(6)?,
                mime_type: row.get(7)?,
                uploaded_at: row.get(8)?,
            })
        })
        .map_err(|e| format!("查询文件失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析文件数据失败: {}", e))?;

    Ok(files)
}

/// 读取文件内容（Base64），用于前端预览
#[tauri::command]
pub fn read_file_base64(file_id: String, db: State<Database>, app_dir: State<AppDir>) -> Result<String, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    let (stored_path, mime_type): (String, String) = conn
        .query_row(
            "SELECT stored_path, mime_type FROM checkup_files WHERE id = ?1",
            [&file_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("文件不存在: {}", e))?;

    let full_path = app_dir.0.join(&stored_path);
    let bytes = std::fs::read(&full_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    Ok(format!("data:{};base64,{}", mime_type, b64))
}

/// 删除文件
#[tauri::command]
pub fn delete_file(file_id: String, db: State<Database>, app_dir: State<AppDir>) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    let stored_path: String = conn
        .query_row(
            "SELECT stored_path FROM checkup_files WHERE id = ?1",
            [&file_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("文件不存在: {}", e))?;

    // 删除关联的 OCR 结果和指标值
    conn.execute("DELETE FROM indicator_values WHERE ocr_result_id IN (SELECT id FROM ocr_results WHERE file_id = ?1)", [&file_id]).ok();
    conn.execute("DELETE FROM ocr_results WHERE file_id = ?1", [&file_id]).ok();

    // 删除数据库记录
    conn.execute("DELETE FROM checkup_files WHERE id = ?1", [&file_id])
        .map_err(|e| format!("删除文件记录失败: {}", e))?;

    // 删除物理文件
    let full_path = app_dir.0.join(&stored_path);
    std::fs::remove_file(&full_path).ok();

    Ok(true)
}

/// 根据文件扩展名推断 MIME 类型
fn guess_mime_type(filename: &str) -> String {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "png" => "image/png".to_string(),
        "gif" => "image/gif".to_string(),
        "bmp" => "image/bmp".to_string(),
        "webp" => "image/webp".to_string(),
        "pdf" => "application/pdf".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}
