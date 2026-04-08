use super::scope::{resolve_member_scope, MemberScopeInput, ResolvedMemberScope};
use crate::db::Database;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckupRecord {
    pub id: String,
    pub checkup_date: String,
    pub status: String,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
    /// 关联的文件数量（查询时填充）
    pub file_count: Option<i32>,
    /// 关联的项目名称列表（查询时填充）
    pub project_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRecordInput {
    pub checkup_date: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecordInput {
    pub id: String,
    pub checkup_date: Option<String>,
    pub notes: Option<String>,
    pub status: Option<String>,
}

fn list_records_with_conn(
    conn: &Connection,
    limit: Option<i64>,
    offset: Option<i64>,
    scope: &ResolvedMemberScope,
) -> Result<Vec<CheckupRecord>, String> {
    let limit_val = limit.unwrap_or(1000); // Default to a large number if not specified to maintain "list all" behavior roughly, or just update callers
    let offset_val = offset.unwrap_or(0);

    let mut stmt = conn
        .prepare(
            "SELECT r.id, r.checkup_date, r.status, r.notes, r.created_at, r.updated_at,
                    (SELECT COUNT(*) FROM checkup_files WHERE record_id = r.id AND owner_user_id = r.owner_user_id AND member_id = r.member_id) as file_count
             FROM checkup_records r
             WHERE r.owner_user_id = ?1 AND r.member_id = ?2
             ORDER BY r.checkup_date DESC, r.created_at DESC
             LIMIT ?3 OFFSET ?4",
        )
        .map_err(|e| format!("查询检查记录失败: {}", e))?;

    let records = stmt
        .query_map(
            rusqlite::params![
                &scope.owner_user_id,
                &scope.member_id,
                limit_val,
                offset_val
            ],
            |row| {
            let record_id: String = row.get(0)?;
            Ok((
                record_id.clone(),
                CheckupRecord {
                    id: record_id,
                    checkup_date: row.get(1)?,
                    status: row.get(2)?,
                    notes: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                    file_count: Some(row.get(6)?),
                    project_names: None,
                },
            ))
        })
        .map_err(|e| format!("查询失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析失败: {}", e))?;

    // 为每条记录查询关联的项目名称
    let mut result: Vec<CheckupRecord> = Vec::new();
    for (record_id, mut record) in records {
        let mut pstmt = conn
            .prepare(
                "SELECT DISTINCT p.name FROM checkup_files f
                 JOIN checkup_projects p ON f.project_id = p.id
                 WHERE f.record_id = ?1 AND f.owner_user_id = ?2 AND f.member_id = ?3",
            )
            .map_err(|e| format!("查询项目名称失败: {}", e))?;
        let names: Vec<String> = pstmt
            .query_map(
                rusqlite::params![&record_id, &scope.owner_user_id, &scope.member_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("查询项目名称失败: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_default();
        record.project_names = Some(names);
        result.push(record);
    }

    Ok(result)
}

/// 查询全部检查记录（倒序），支持分页
#[tauri::command]
pub fn list_records(
    limit: Option<i64>,
    offset: Option<i64>,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<Vec<CheckupRecord>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    list_records_with_conn(conn, limit, offset, &scope)
}

fn create_record_with_conn(
    conn: &Connection,
    input: CreateRecordInput,
    scope: &ResolvedMemberScope,
) -> Result<CheckupRecord, String> {
    let now = chrono::Local::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();
    let notes = input.notes.unwrap_or_default();

    conn.execute(
        "INSERT INTO checkup_records (id, owner_user_id, member_id, member_name_snapshot, checkup_date, status, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'pending_upload', ?6, ?7, ?8)",
        rusqlite::params![
            id,
            &scope.owner_user_id,
            &scope.member_id,
            &scope.member_name,
            input.checkup_date,
            notes,
            now,
            now
        ],
    )
    .map_err(|e| format!("创建检查记录失败: {}", e))?;

    Ok(CheckupRecord {
        id,
        checkup_date: input.checkup_date,
        status: "pending_upload".to_string(),
        notes,
        created_at: now.clone(),
        updated_at: now,
        file_count: Some(0),
        project_names: Some(vec![]),
    })
}

/// 创建检查记录
#[tauri::command]
pub fn create_record(
    input: CreateRecordInput,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<CheckupRecord, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    create_record_with_conn(conn, input, &scope)
}

fn update_record_with_conn(
    conn: &Connection,
    input: UpdateRecordInput,
    scope: &ResolvedMemberScope,
) -> Result<bool, String> {
    let now = chrono::Local::now().to_rfc3339();

    let existing = conn
        .query_row(
            "SELECT checkup_date, notes, status
             FROM checkup_records
             WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
            rusqlite::params![&input.id, &scope.owner_user_id, &scope.member_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .map_err(|e| format!("记录不存在: {}", e))?;

    let date = input.checkup_date.unwrap_or(existing.0);
    let notes = input.notes.unwrap_or(existing.1);
    let status = input.status.unwrap_or(existing.2);

    conn.execute(
        "UPDATE checkup_records
         SET checkup_date=?1, notes=?2, status=?3, updated_at=?4
         WHERE id=?5 AND owner_user_id=?6 AND member_id=?7",
        rusqlite::params![
            date,
            notes,
            status,
            now,
            input.id,
            &scope.owner_user_id,
            &scope.member_id
        ],
    )
    .map_err(|e| format!("更新记录失败: {}", e))?;

    Ok(true)
}

/// 更新检查记录
#[tauri::command]
pub fn update_record(
    input: UpdateRecordInput,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    update_record_with_conn(conn, input, &scope)
}

fn delete_record_with_conn(
    conn: &Connection,
    id: String,
    scope: &ResolvedMemberScope,
) -> Result<bool, String> {
    // 级联删除：indicator_values -> ocr_results -> ai_analyses -> checkup_files -> checkup_records
    conn.execute(
        "DELETE FROM indicator_values WHERE record_id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
        rusqlite::params![&id, &scope.owner_user_id, &scope.member_id],
    )
        .map_err(|e| format!("删除指标值失败: {}", e))?;
    conn.execute(
        "DELETE FROM ocr_results WHERE record_id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
        rusqlite::params![&id, &scope.owner_user_id, &scope.member_id],
    )
        .map_err(|e| format!("删除OCR结果失败: {}", e))?;
    conn.execute(
        "DELETE FROM ai_analyses WHERE record_id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
        rusqlite::params![&id, &scope.owner_user_id, &scope.member_id],
    )
        .map_err(|e| format!("删除AI分析失败: {}", e))?;
    conn.execute(
        "DELETE FROM checkup_files WHERE record_id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
        rusqlite::params![&id, &scope.owner_user_id, &scope.member_id],
    )
        .map_err(|e| format!("删除文件记录失败: {}", e))?;
    conn.execute(
        "DELETE FROM checkup_records WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
        rusqlite::params![&id, &scope.owner_user_id, &scope.member_id],
    )
        .map_err(|e| format!("删除检查记录失败: {}", e))?;

    Ok(true)
}

/// 删除检查记录（级联删除关联数据）
#[tauri::command]
pub fn delete_record(
    id: String,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    delete_record_with_conn(conn, id, &scope)
}

use crate::commands::ocr::OcrParsedItem;

#[derive(Debug, Serialize, Clone)]
pub struct HistoryTimelineItem {
    pub id: String,
    pub checkup_date: String,
    pub status: String,
    pub notes: String,
    pub abnormal_items: Vec<AbnormalItem>,
    pub ai_analysis: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AbnormalItem {
    pub project_name: String,
    pub name: String,
    pub value: String,
    pub unit: String,
    pub reference_range: String,
}

fn get_history_timeline_with_conn(
    conn: &Connection,
    limit: Option<i64>,
    offset: Option<i64>,
    scope: &ResolvedMemberScope,
) -> Result<Vec<HistoryTimelineItem>, String> {
    let limit_val = limit.unwrap_or(10);
    let offset_val = offset.unwrap_or(0);

    // 1. 获取分页检查记录
    let mut stmt = conn
        .prepare(
            "SELECT id, checkup_date, status, notes 
         FROM checkup_records 
         WHERE owner_user_id = ?1 AND member_id = ?2
         ORDER BY checkup_date DESC, created_at DESC
         LIMIT ?3 OFFSET ?4",
        )
        .map_err(|e| format!("查询记录失败: {}", e))?;

    let records = stmt
        .query_map(
            rusqlite::params![
                &scope.owner_user_id,
                &scope.member_id,
                limit_val,
                offset_val
            ],
            |row| {
            Ok((
                row.get::<_, String>(0)?, // id
                row.get::<_, String>(1)?, // date
                row.get::<_, String>(2)?, // status
                row.get::<_, String>(3)?, // notes
            ))
        })
        .map_err(|e| format!("查询失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析失败: {}", e))?;

    let mut result = Vec::new();

    for (id, date, status, notes) in records {
        // 2. 获取该记录的异常指标 (通过 OCR 结果)
        let mut abnormal_items = Vec::new();
        let mut ocr_stmt = conn
            .prepare(
                "SELECT o.parsed_items, p.name 
              FROM ocr_results o
              LEFT JOIN checkup_projects p ON o.project_id = p.id
              WHERE o.record_id = ?1
                AND o.owner_user_id = ?2
                AND o.member_id = ?3
                AND o.status = 'success'",
            )
            .map_err(|e| format!("查询OCR失败: {}", e))?;

        let ocr_rows = ocr_stmt
            .query_map(
                rusqlite::params![&id, &scope.owner_user_id, &scope.member_id],
                |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1).unwrap_or_default(),
                ))
            })
            .map_err(|e| format!("查询OCR失败: {}", e))?;

        for row in ocr_rows {
            if let Ok((json, project_name)) = row {
                if let Ok(items) = serde_json::from_str::<Vec<OcrParsedItem>>(&json) {
                    for item in items {
                        if item.is_abnormal {
                            abnormal_items.push(AbnormalItem {
                                project_name: project_name.clone(),
                                name: item.name,
                                value: item.value,
                                unit: item.unit,
                                reference_range: item.reference_range,
                            });
                        }
                    }
                }
            }
        }

        // 3. 获取最新的 AI 分析结果
        let ai_analysis: Option<String> = conn.query_row(
            "SELECT response_content
             FROM ai_analyses
             WHERE record_id = ?1
               AND owner_user_id = ?2
               AND member_id = ?3
               AND status = 'success'
             ORDER BY created_at DESC
             LIMIT 1",
            rusqlite::params![&id, &scope.owner_user_id, &scope.member_id],
            |row| row.get(0)
        ).ok();

        result.push(HistoryTimelineItem {
            id,
            checkup_date: date,
            status,
            notes,
            abnormal_items,
            ai_analysis,
        });
    }

    Ok(result)
}
#[tauri::command]
pub fn get_history_timeline(
    limit: Option<i64>,
    offset: Option<i64>,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<Vec<HistoryTimelineItem>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    get_history_timeline_with_conn(conn, limit, offset, &scope)
}
/// 获取单条检查记录详情
fn get_record_with_conn(
    conn: &Connection,
    id: String,
    scope: &ResolvedMemberScope,
) -> Result<CheckupRecord, String> {
    let mut record = conn
        .query_row(
            "SELECT id, checkup_date, status, notes, created_at, updated_at,
                (SELECT COUNT(*) FROM checkup_files WHERE record_id = ?1 AND owner_user_id = ?2 AND member_id = ?3) as file_count
         FROM checkup_records
         WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
            rusqlite::params![&id, &scope.owner_user_id, &scope.member_id],
            |row| {
                Ok(CheckupRecord {
                    id: row.get(0)?,
                    checkup_date: row.get(1)?,
                    status: row.get(2)?,
                    notes: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                    file_count: Some(row.get(6)?),
                    project_names: None,
                })
            },
        )
        .map_err(|e| format!("记录不存在: {}", e))?;

    // 查询关联项目名称
    let mut pstmt = conn
        .prepare(
            "SELECT DISTINCT p.name FROM checkup_files f
             JOIN checkup_projects p ON f.project_id = p.id
             WHERE f.record_id = ?1 AND f.owner_user_id = ?2 AND f.member_id = ?3",
        )
        .map_err(|e| format!("查询失败: {}", e))?;
    let names: Vec<String> = pstmt
        .query_map(
            rusqlite::params![&id, &scope.owner_user_id, &scope.member_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_default();
    record.project_names = Some(names);

    Ok(record)
}

/// 获取单条检查记录详情
#[tauri::command]
pub fn get_record(
    id: String,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<CheckupRecord, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    get_record_with_conn(conn, id, &scope)
}

fn get_or_create_today_record_with_conn(
    conn: &Connection,
    scope: &ResolvedMemberScope,
) -> Result<CheckupRecord, String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id
             FROM checkup_records
             WHERE checkup_date = ?1 AND owner_user_id = ?2 AND member_id = ?3
             LIMIT 1",
            rusqlite::params![&today, &scope.owner_user_id, &scope.member_id],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing_id {
        return get_record_with_conn(conn, id, scope);
    }

    let input = CreateRecordInput {
        checkup_date: today,
        notes: None,
    };
    create_record_with_conn(conn, input, scope)
}

#[tauri::command]
pub fn get_or_create_today_record(
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<CheckupRecord, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    get_or_create_today_record_with_conn(conn, &scope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_database(name: &str) -> (Database, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "health-monitor-record-tests-{}-{}",
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

    fn member_scope(owner_user_id: &str, member_id: &str, member_name: &str) -> ResolvedMemberScope {
        ResolvedMemberScope {
            owner_user_id: owner_user_id.to_string(),
            member_id: member_id.to_string(),
            member_name: member_name.to_string(),
        }
    }

    fn seed_project(conn: &Connection) {
        conn.execute(
            "INSERT INTO checkup_projects (id, name, description, sort_order, is_active, created_at, updated_at)
             VALUES ('proj-blood', '血常规', '', 0, 1, '2026-04-08T00:00:00+08:00', '2026-04-08T00:00:00+08:00')",
            [],
        )
        .expect("project should seed");
        conn.execute(
            "INSERT INTO indicators (id, project_id, name, unit, reference_range, sort_order, is_core, created_at)
             VALUES ('ind-wbc', 'proj-blood', '白细胞', '10^9/L', '3.5-9.5', 0, 1, '2026-04-08T00:00:00+08:00')",
            [],
        )
        .expect("indicator should seed");
    }

    fn seed_member_record_bundle(
        conn: &Connection,
        owner_user_id: &str,
        member_id: &str,
        record_id: &str,
        file_id: &str,
        ocr_id: &str,
        ai_id: &str,
        indicator_value_id: &str,
        checkup_date: &str,
        analysis_text: &str,
    ) {
        let abnormal_items = serde_json::to_string(&vec![OcrParsedItem {
            name: "白细胞".to_string(),
            value: "11.2".to_string(),
            unit: "10^9/L".to_string(),
            reference_range: "3.5-9.5".to_string(),
            is_abnormal: true,
        }])
        .expect("json should serialize");

        conn.execute(
            "INSERT INTO checkup_records (
                id, owner_user_id, member_id, member_name_snapshot, checkup_date, status, notes, created_at, updated_at
             ) VALUES (?1, ?2, ?3, '成员', ?4, 'ocr_done', '备注', '2026-04-08T00:00:00+08:00', '2026-04-08T00:00:00+08:00')",
            rusqlite::params![record_id, owner_user_id, member_id, checkup_date],
        )
        .expect("record should seed");
        conn.execute(
            "INSERT INTO checkup_files (
                id, owner_user_id, member_id, record_id, project_id, original_filename, stored_path, file_size, mime_type, uploaded_at
             ) VALUES (?1, ?2, ?3, ?4, 'proj-blood', 'report.png', 'pictures/report.png', 123, 'image/png', '2026-04-08T00:00:00+08:00')",
            rusqlite::params![file_id, owner_user_id, member_id, record_id],
        )
        .expect("file should seed");
        conn.execute(
            "INSERT INTO ocr_results (
                id, file_id, record_id, project_id, owner_user_id, member_id, checkup_date, raw_json, parsed_items, status, error_message, created_at
             ) VALUES (?1, ?2, ?3, 'proj-blood', ?4, ?5, ?6, '{}', ?7, 'success', '', '2026-04-08T00:00:00+08:00')",
            rusqlite::params![ocr_id, file_id, record_id, owner_user_id, member_id, checkup_date, abnormal_items],
        )
        .expect("ocr should seed");
        conn.execute(
            "INSERT INTO ai_analyses (
                id, owner_user_id, member_id, record_id, request_prompt, response_content, model_used, status, error_message, created_at
             ) VALUES (?1, ?2, ?3, ?4, '', ?5, 'gpt-test', 'success', '', '2026-04-08T00:00:00+08:00')",
            rusqlite::params![ai_id, owner_user_id, member_id, record_id, analysis_text],
        )
        .expect("analysis should seed");
        conn.execute(
            "INSERT INTO indicator_values (
                id, ocr_result_id, record_id, project_id, indicator_id, owner_user_id, member_id, checkup_date, value, value_text, is_abnormal, created_at
             ) VALUES (?1, ?2, ?3, 'proj-blood', 'ind-wbc', ?4, ?5, ?6, 11.2, '11.2', 1, '2026-04-08T00:00:00+08:00')",
            rusqlite::params![indicator_value_id, ocr_id, record_id, owner_user_id, member_id, checkup_date],
        )
        .expect("indicator value should seed");
    }

    #[test]
    fn list_records_only_returns_current_member_rows() {
        let (db, dir) = create_test_database("list-records");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");
            seed_project(conn);
            seed_member_record_bundle(
                conn,
                "user-1",
                "member-a",
                "record-a",
                "file-a",
                "ocr-a",
                "ai-a",
                "iv-a",
                "2026-04-07",
                "成员A分析",
            );
            seed_member_record_bundle(
                conn,
                "user-1",
                "member-b",
                "record-b",
                "file-b",
                "ocr-b",
                "ai-b",
                "iv-b",
                "2026-04-07",
                "成员B分析",
            );

            let records = list_records_with_conn(conn, Some(20), Some(0), &member_scope("user-1", "member-a", "本人"))
                .expect("member A records should load");

            assert_eq!(records.len(), 1);
            assert_eq!(records[0].id, "record-a");
            assert_eq!(records[0].file_count, Some(1));
            assert_eq!(
                records[0].project_names.as_ref().expect("project names should exist"),
                &vec!["血常规".to_string()]
            );
        }
        cleanup_test_database(&db, dir);
    }

    #[test]
    fn history_timeline_only_aggregates_current_member_analysis_and_abnormal_items() {
        let (db, dir) = create_test_database("history-timeline");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");
            seed_project(conn);
            seed_member_record_bundle(
                conn,
                "user-1",
                "member-a",
                "record-a",
                "file-a",
                "ocr-a",
                "ai-a",
                "iv-a",
                "2026-04-07",
                "成员A分析",
            );
            seed_member_record_bundle(
                conn,
                "user-1",
                "member-b",
                "record-b",
                "file-b",
                "ocr-b",
                "ai-b",
                "iv-b",
                "2026-04-07",
                "成员B分析",
            );

            let timeline = get_history_timeline_with_conn(
                conn,
                Some(10),
                Some(0),
                &member_scope("user-1", "member-a", "本人"),
            )
            .expect("timeline should load");

            assert_eq!(timeline.len(), 1);
            assert_eq!(timeline[0].id, "record-a");
            assert_eq!(timeline[0].ai_analysis.as_deref(), Some("成员A分析"));
            assert_eq!(timeline[0].abnormal_items.len(), 1);
            assert_eq!(timeline[0].abnormal_items[0].project_name, "血常规");
            assert_eq!(timeline[0].abnormal_items[0].name, "白细胞");
        }
        cleanup_test_database(&db, dir);
    }

    #[test]
    fn delete_record_only_cascades_inside_current_member_scope() {
        let (db, dir) = create_test_database("delete-record");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");
            seed_project(conn);
            seed_member_record_bundle(
                conn,
                "user-1",
                "member-a",
                "record-a",
                "file-a",
                "ocr-a",
                "ai-a",
                "iv-a",
                "2026-04-07",
                "成员A分析",
            );
            seed_member_record_bundle(
                conn,
                "user-1",
                "member-b",
                "record-b",
                "file-b",
                "ocr-b",
                "ai-b",
                "iv-b",
                "2026-04-07",
                "成员B分析",
            );

            delete_record_with_conn(conn, "record-a".to_string(), &member_scope("user-1", "member-a", "本人"))
                .expect("member A record should delete");

            let remaining_member_a_records: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM checkup_records WHERE owner_user_id = 'user-1' AND member_id = 'member-a'",
                    [],
                    |row| row.get(0),
                )
                .expect("count should work");
            let remaining_member_b_records: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM checkup_records WHERE owner_user_id = 'user-1' AND member_id = 'member-b'",
                    [],
                    |row| row.get(0),
                )
                .expect("count should work");
            let remaining_member_b_files: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM checkup_files WHERE owner_user_id = 'user-1' AND member_id = 'member-b'",
                    [],
                    |row| row.get(0),
                )
                .expect("count should work");

            assert_eq!(remaining_member_a_records, 0);
            assert_eq!(remaining_member_b_records, 1);
            assert_eq!(remaining_member_b_files, 1);
        }
        cleanup_test_database(&db, dir);
    }
}
