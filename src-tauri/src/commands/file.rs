use super::AppDir;
use super::scope::{MemberScopeInput, ResolvedMemberScope, resolve_member_scope};
use crate::db::Database;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

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
fn upload_files_with_conn(
    conn: &Connection,
    files: Vec<UploadFileInput>,
    scope: &ResolvedMemberScope,
    app_dir: &Path,
) -> Result<Vec<CheckupFile>, String> {
    let now = chrono::Local::now().to_rfc3339();
    let mut result = Vec::new();

    for file_input in files {
        let id = uuid::Uuid::new_v4().to_string();

        // 获取项目名称用于目录结构
        let project_name: String = conn
            .query_row(
                "SELECT name
                 FROM checkup_projects
                 WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
                rusqlite::params![
                    &file_input.project_id,
                    &scope.owner_user_id,
                    &scope.member_id
                ],
                |row| row.get(0),
            )
            .map_err(|e| format!("项目不存在或不属于当前成员: {}", e))?;

        let record_exists: bool = conn
            .query_row(
                "SELECT EXISTS(
                    SELECT 1
                    FROM checkup_records
                    WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3
                )",
                rusqlite::params![
                    &file_input.record_id,
                    &scope.owner_user_id,
                    &scope.member_id
                ],
                |row| row.get::<_, i32>(0),
            )
            .map_err(|e| format!("校验检查记录失败: {}", e))?
            == 1;

        if !record_exists {
            return Err("检查记录不存在或不属于当前成员".to_string());
        }

        // 构建存储路径: pictures/<项目名>/<日期>/<文件名>
        let store_dir = app_dir
            .join("pictures")
            .join(&scope.owner_user_id)
            .join(&scope.member_id)
            .join(&project_name)
            .join(&file_input.checkup_date);

        std::fs::create_dir_all(&store_dir).map_err(|e| format!("创建目录失败: {}", e))?;

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

        std::fs::write(&stored_path, &file_bytes).map_err(|e| format!("文件保存失败: {}", e))?;

        // 获取相对路径
        let relative_path = stored_path
            .strip_prefix(app_dir)
            .unwrap_or(&stored_path)
            .to_string_lossy()
            .to_string();

        // 推断 MIME 类型
        let mime_type = guess_mime_type(&file_input.filename);

        // 插入数据库
        conn.execute(
            "INSERT INTO checkup_files (id, owner_user_id, member_id, record_id, project_id, original_filename, stored_path, file_size, mime_type, uploaded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                id,
                &scope.owner_user_id,
                &scope.member_id,
                file_input.record_id,
                file_input.project_id,
                file_input.filename,
                relative_path,
                file_size,
                mime_type,
                now
            ],
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
            "UPDATE checkup_records
             SET status = 'pending_ocr', updated_at = ?1
             WHERE id = ?2 AND owner_user_id = ?3 AND member_id = ?4 AND status = 'pending_upload'",
            rusqlite::params![now, first.record_id, &scope.owner_user_id, &scope.member_id],
        )
        .ok();
    }

    Ok(result)
}

/// 批量上传文件
#[tauri::command]
pub fn upload_files(
    files: Vec<UploadFileInput>,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
    app_dir: State<AppDir>,
) -> Result<Vec<CheckupFile>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    upload_files_with_conn(conn, files, &scope, &app_dir.0)
}

/// 获取某次检查记录的所有文件
fn list_files_with_conn(
    conn: &Connection,
    record_id: String,
    scope: &ResolvedMemberScope,
) -> Result<Vec<CheckupFile>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT f.id, f.record_id, f.project_id, p.name, f.original_filename, f.stored_path, f.file_size, f.mime_type, f.uploaded_at
             FROM checkup_files f
             LEFT JOIN checkup_projects p
                    ON f.project_id = p.id
                   AND p.owner_user_id = f.owner_user_id
                   AND p.member_id = f.member_id
             WHERE f.record_id = ?1 AND f.owner_user_id = ?2 AND f.member_id = ?3
             ORDER BY p.name ASC, f.uploaded_at ASC"
        )
        .map_err(|e| format!("查询文件失败: {}", e))?;

    let files = stmt
        .query_map(
            rusqlite::params![&record_id, &scope.owner_user_id, &scope.member_id],
            |row| {
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
            },
        )
        .map_err(|e| format!("查询文件失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析文件数据失败: {}", e))?;

    Ok(files)
}

/// 获取某次检查记录的所有文件
#[tauri::command]
pub fn list_files(
    record_id: String,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<Vec<CheckupFile>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    list_files_with_conn(conn, record_id, &scope)
}

/// 读取文件内容（Base64），用于前端预览
fn read_file_base64_with_conn(
    conn: &Connection,
    file_id: String,
    scope: &ResolvedMemberScope,
    app_dir: &Path,
) -> Result<String, String> {
    let (stored_path, mime_type): (String, String) = conn
        .query_row(
            "SELECT stored_path, mime_type
             FROM checkup_files
             WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
            rusqlite::params![&file_id, &scope.owner_user_id, &scope.member_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("文件不存在: {}", e))?;

    let full_path = app_dir.join(&stored_path);
    let bytes = std::fs::read(&full_path).map_err(|e| format!("读取文件失败: {}", e))?;

    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    Ok(format!("data:{};base64,{}", mime_type, b64))
}

/// 读取文件内容（Base64），用于前端预览
#[tauri::command]
pub fn read_file_base64(
    file_id: String,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
    app_dir: State<AppDir>,
) -> Result<String, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    read_file_base64_with_conn(conn, file_id, &scope, &app_dir.0)
}

/// 删除文件
fn delete_file_with_conn(
    conn: &Connection,
    file_id: String,
    scope: &ResolvedMemberScope,
    app_dir: &Path,
) -> Result<bool, String> {
    let stored_path: String = conn
        .query_row(
            "SELECT stored_path
             FROM checkup_files
             WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
            rusqlite::params![&file_id, &scope.owner_user_id, &scope.member_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("文件不存在: {}", e))?;

    // 删除关联的 OCR 结果和指标值
    conn.execute(
        "DELETE FROM indicator_values
         WHERE ocr_result_id IN (
            SELECT id FROM ocr_results WHERE file_id = ?1 AND owner_user_id = ?2 AND member_id = ?3
         )",
        rusqlite::params![&file_id, &scope.owner_user_id, &scope.member_id],
    )
    .ok();
    conn.execute(
        "DELETE FROM ocr_results WHERE file_id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
        rusqlite::params![&file_id, &scope.owner_user_id, &scope.member_id],
    )
    .ok();

    // 删除数据库记录
    conn.execute(
        "DELETE FROM checkup_files WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
        rusqlite::params![&file_id, &scope.owner_user_id, &scope.member_id],
    )
    .map_err(|e| format!("删除文件记录失败: {}", e))?;

    // 删除物理文件
    let full_path = app_dir.join(&stored_path);
    std::fs::remove_file(&full_path).ok();

    Ok(true)
}

/// 删除文件
#[tauri::command]
pub fn delete_file(
    file_id: String,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
    app_dir: State<AppDir>,
) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    delete_file_with_conn(conn, file_id, &scope, &app_dir.0)
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

/// 读取临时文件（用于手机上传预览）
#[tauri::command]
pub fn read_temp_file(path: String) -> Result<String, String> {
    // 读取文件并转为 Base64
    let bytes = std::fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))?;
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    let mime = guess_mime_type(&path);
    Ok(format!("data:{};base64,{}", mime, b64))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_database(name: &str) -> (Database, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "health-monitor-file-tests-{}-{}",
            name,
            uuid::Uuid::new_v4()
        ));
        let db = Database::new(dir.clone()).expect("database should initialize");
        (db, dir)
    }

    fn cleanup_test_database(db: &Database, dir: PathBuf) {
        db.close().ok();
        fs::remove_dir_all(dir).ok();
    }

    fn member_scope(
        owner_user_id: &str,
        member_id: &str,
        member_name: &str,
    ) -> ResolvedMemberScope {
        ResolvedMemberScope {
            owner_user_id: owner_user_id.to_string(),
            member_id: member_id.to_string(),
            member_name: member_name.to_string(),
        }
    }

    fn seed_project(conn: &Connection, owner_user_id: &str, member_id: &str) {
        conn.execute(
            "INSERT INTO checkup_projects (id, owner_user_id, member_id, name, description, sort_order, is_active, created_at, updated_at)
             VALUES ('proj-blood', ?1, ?2, '血常规', '', 0, 1, '2026-04-08T00:00:00+08:00', '2026-04-08T00:00:00+08:00')",
            rusqlite::params![owner_user_id, member_id],
        )
        .expect("project should seed");
    }

    fn seed_record(conn: &Connection, owner_user_id: &str, member_id: &str, record_id: &str) {
        conn.execute(
            "INSERT INTO checkup_records (
                id, owner_user_id, member_id, member_name_snapshot, checkup_date, status, notes, created_at, updated_at
             ) VALUES (?1, ?2, ?3, '成员', '2026-04-08', 'pending_upload', '', '2026-04-08T00:00:00+08:00', '2026-04-08T00:00:00+08:00')",
            rusqlite::params![record_id, owner_user_id, member_id],
        )
        .expect("record should seed");
    }

    fn seed_file(
        conn: &Connection,
        owner_user_id: &str,
        member_id: &str,
        record_id: &str,
        file_id: &str,
        stored_path: &str,
    ) {
        conn.execute(
            "INSERT INTO checkup_files (
                id, owner_user_id, member_id, record_id, project_id, original_filename, stored_path, file_size, mime_type, uploaded_at
             ) VALUES (?1, ?2, ?3, ?4, 'proj-blood', 'report.png', ?5, 3, 'image/png', '2026-04-08T00:00:00+08:00')",
            rusqlite::params![file_id, owner_user_id, member_id, record_id, stored_path],
        )
        .expect("file should seed");
    }

    #[test]
    fn upload_and_list_files_only_operate_inside_current_member_scope() {
        let (db, dir) = create_test_database("upload-list");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");
            seed_project(conn, "user-1", "member-a");
            seed_record(conn, "user-1", "member-a", "record-a");
            seed_record(conn, "user-1", "member-b", "record-b");

            let uploaded = upload_files_with_conn(
                conn,
                vec![UploadFileInput {
                    record_id: "record-a".to_string(),
                    project_id: "proj-blood".to_string(),
                    checkup_date: "2026-04-08".to_string(),
                    file_data: base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        b"abc",
                    ),
                    filename: "report.png".to_string(),
                }],
                &member_scope("user-1", "member-a", "本人"),
                &dir,
            )
            .expect("upload should succeed");

            assert_eq!(uploaded.len(), 1);

            let files_a = list_files_with_conn(
                conn,
                "record-a".to_string(),
                &member_scope("user-1", "member-a", "本人"),
            )
            .expect("member A files should load");
            let files_b = list_files_with_conn(
                conn,
                "record-a".to_string(),
                &member_scope("user-1", "member-b", "母亲"),
            )
            .expect("member B files should load");

            assert_eq!(files_a.len(), 1);
            assert_eq!(files_b.len(), 0);
            assert!(dir.join(&files_a[0].stored_path).exists());
        }
        cleanup_test_database(&db, dir);
    }

    #[test]
    fn read_file_base64_rejects_cross_member_access() {
        let (db, dir) = create_test_database("read-file");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");
            seed_project(conn, "user-1", "member-a");
            seed_record(conn, "user-1", "member-a", "record-a");
            let relative_path = "pictures/user-1/member-a/血常规/2026-04-08/file-a.png";
            let full_path = dir.join(relative_path);
            fs::create_dir_all(full_path.parent().expect("parent should exist"))
                .expect("directory should exist");
            fs::write(&full_path, b"abc").expect("file should write");
            seed_file(
                conn,
                "user-1",
                "member-a",
                "record-a",
                "file-a",
                relative_path,
            );

            let preview = read_file_base64_with_conn(
                conn,
                "file-a".to_string(),
                &member_scope("user-1", "member-a", "本人"),
                &dir,
            )
            .expect("owner should read file");
            let denied = read_file_base64_with_conn(
                conn,
                "file-a".to_string(),
                &member_scope("user-1", "member-b", "母亲"),
                &dir,
            );

            assert!(preview.starts_with("data:image/png;base64,"));
            assert!(denied.is_err());
        }
        cleanup_test_database(&db, dir);
    }

    #[test]
    fn delete_file_only_removes_current_member_file_and_ocr_chain() {
        let (db, dir) = create_test_database("delete-file");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");
            seed_project(conn, "user-1", "member-a");
            conn.execute(
                "INSERT INTO indicators (id, project_id, name, unit, reference_range, sort_order, is_core, created_at)
                 VALUES ('ind-wbc', 'proj-blood', '白细胞', '10^9/L', '3.5-9.5', 0, 1, '2026-04-08T00:00:00+08:00')",
                [],
            )
            .expect("indicator should seed");
            seed_record(conn, "user-1", "member-a", "record-a");
            seed_record(conn, "user-1", "member-b", "record-b");

            let relative_a = "pictures/user-1/member-a/血常规/2026-04-08/file-a.png";
            let relative_b = "pictures/user-1/member-b/血常规/2026-04-08/file-b.png";
            for relative in [relative_a, relative_b] {
                let path = dir.join(relative);
                fs::create_dir_all(path.parent().expect("parent should exist"))
                    .expect("directory should exist");
                fs::write(path, b"abc").expect("file should write");
            }

            seed_file(conn, "user-1", "member-a", "record-a", "file-a", relative_a);
            seed_file(conn, "user-1", "member-b", "record-b", "file-b", relative_b);
            conn.execute(
                "INSERT INTO ocr_results (
                    id, file_id, record_id, project_id, owner_user_id, member_id, checkup_date, raw_json, parsed_items, status, error_message, created_at
                 ) VALUES
                    ('ocr-a', 'file-a', 'record-a', 'proj-blood', 'user-1', 'member-a', '2026-04-08', '{}', '[]', 'success', '', '2026-04-08T00:00:00+08:00'),
                    ('ocr-b', 'file-b', 'record-b', 'proj-blood', 'user-1', 'member-b', '2026-04-08', '{}', '[]', 'success', '', '2026-04-08T00:00:00+08:00')",
                [],
            )
            .expect("ocr should seed");
            conn.execute(
                "INSERT INTO indicator_values (
                    id, ocr_result_id, record_id, project_id, indicator_id, owner_user_id, member_id, checkup_date, value, value_text, is_abnormal, created_at
                 ) VALUES
                    ('iv-a', 'ocr-a', 'record-a', 'proj-blood', 'ind-wbc', 'user-1', 'member-a', '2026-04-08', 11.2, '11.2', 1, '2026-04-08T00:00:00+08:00'),
                    ('iv-b', 'ocr-b', 'record-b', 'proj-blood', 'ind-wbc', 'user-1', 'member-b', '2026-04-08', 15.0, '15.0', 1, '2026-04-08T00:00:00+08:00')",
                [],
            )
            .expect("indicator values should seed");

            delete_file_with_conn(
                conn,
                "file-a".to_string(),
                &member_scope("user-1", "member-a", "本人"),
                &dir,
            )
            .expect("delete should succeed");

            let remaining_member_a_files: i64 = conn.query_row(
                "SELECT COUNT(*) FROM checkup_files WHERE owner_user_id = 'user-1' AND member_id = 'member-a'",
                [],
                |row| row.get(0),
            ).expect("count should work");
            let remaining_member_b_files: i64 = conn.query_row(
                "SELECT COUNT(*) FROM checkup_files WHERE owner_user_id = 'user-1' AND member_id = 'member-b'",
                [],
                |row| row.get(0),
            ).expect("count should work");
            let remaining_member_b_ocr: i64 = conn.query_row(
                "SELECT COUNT(*) FROM ocr_results WHERE owner_user_id = 'user-1' AND member_id = 'member-b'",
                [],
                |row| row.get(0),
            ).expect("count should work");

            assert_eq!(remaining_member_a_files, 0);
            assert_eq!(remaining_member_b_files, 1);
            assert_eq!(remaining_member_b_ocr, 1);
            assert!(!dir.join(relative_a).exists());
            assert!(dir.join(relative_b).exists());
        }
        cleanup_test_database(&db, dir);
    }
}
