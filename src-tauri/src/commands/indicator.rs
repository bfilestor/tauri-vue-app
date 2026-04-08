use super::scope::{MemberScopeInput, ResolvedMemberScope, resolve_member_scope};
use crate::db::Database;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Indicator {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub unit: String,
    pub reference_range: String,
    pub sort_order: i32,
    pub is_core: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateIndicatorInput {
    pub project_id: String,
    pub name: String,
    pub unit: Option<String>,
    pub reference_range: Option<String>,
    pub is_core: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIndicatorInput {
    pub id: String,
    pub name: Option<String>,
    pub unit: Option<String>,
    pub reference_range: Option<String>,
    pub is_core: Option<bool>,
    pub sort_order: Option<i32>,
}

fn ensure_project_in_scope(
    conn: &Connection,
    project_id: &str,
    scope: &ResolvedMemberScope,
) -> Result<(), String> {
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM checkup_projects
                WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3
            )",
            params![project_id, &scope.owner_user_id, &scope.member_id],
            |row| row.get::<_, i32>(0),
        )
        .map_err(|e| format!("校验项目失败: {}", e))?
        == 1;

    if !exists {
        return Err("项目不存在或不属于当前成员".to_string());
    }

    Ok(())
}

fn list_indicators_with_conn(
    conn: &Connection,
    project_id: String,
    scope: &ResolvedMemberScope,
) -> Result<Vec<Indicator>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, name, unit, reference_range, sort_order, is_core, created_at
             FROM indicators
             WHERE project_id = ?1 AND owner_user_id = ?2 AND member_id = ?3
             ORDER BY sort_order ASC, created_at ASC",
        )
        .map_err(|e| format!("查询指标失败: {}", e))?;

    let indicators = stmt
        .query_map(
            params![&project_id, &scope.owner_user_id, &scope.member_id],
            |row| {
                Ok(Indicator {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    unit: row.get(3)?,
                    reference_range: row.get(4)?,
                    sort_order: row.get(5)?,
                    is_core: row.get::<_, i32>(6)? == 1,
                    created_at: row.get(7)?,
                })
            },
        )
        .map_err(|e| format!("查询指标失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析指标数据失败: {}", e))?;

    Ok(indicators)
}

fn create_indicator_with_conn(
    conn: &Connection,
    input: CreateIndicatorInput,
    scope: &ResolvedMemberScope,
) -> Result<Indicator, String> {
    ensure_project_in_scope(conn, &input.project_id, scope)?;

    let now = chrono::Local::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();
    let unit = input.unit.unwrap_or_default();
    let reference_range = input.reference_range.unwrap_or_default();
    let is_core = input.is_core.unwrap_or(false);

    conn.execute(
        "INSERT INTO indicators (
            id, project_id, owner_user_id, member_id, name, unit, reference_range, sort_order, is_core, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9)",
        params![
            id,
            input.project_id,
            &scope.owner_user_id,
            &scope.member_id,
            input.name,
            unit,
            reference_range,
            is_core as i32,
            now
        ],
    )
    .map_err(|e| format!("创建指标失败: {}", e))?;

    Ok(Indicator {
        id,
        project_id: input.project_id,
        name: input.name,
        unit,
        reference_range,
        sort_order: 0,
        is_core,
        created_at: now,
    })
}

fn update_indicator_with_conn(
    conn: &Connection,
    input: UpdateIndicatorInput,
    scope: &ResolvedMemberScope,
) -> Result<Indicator, String> {
    let existing = conn
        .query_row(
            "SELECT id, project_id, name, unit, reference_range, sort_order, is_core, created_at
             FROM indicators
             WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
            params![&input.id, &scope.owner_user_id, &scope.member_id],
            |row| {
                Ok(Indicator {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    unit: row.get(3)?,
                    reference_range: row.get(4)?,
                    sort_order: row.get(5)?,
                    is_core: row.get::<_, i32>(6)? == 1,
                    created_at: row.get(7)?,
                })
            },
        )
        .map_err(|e| format!("指标不存在或不属于当前成员: {}", e))?;

    let name = input.name.unwrap_or(existing.name);
    let unit = input.unit.unwrap_or(existing.unit);
    let reference_range = input.reference_range.unwrap_or(existing.reference_range);
    let is_core = input.is_core.unwrap_or(existing.is_core);
    let sort_order = input.sort_order.unwrap_or(existing.sort_order);

    conn.execute(
        "UPDATE indicators
         SET name = ?1, unit = ?2, reference_range = ?3, is_core = ?4, sort_order = ?5
         WHERE id = ?6 AND owner_user_id = ?7 AND member_id = ?8",
        params![
            name,
            unit,
            reference_range,
            is_core as i32,
            sort_order,
            input.id,
            &scope.owner_user_id,
            &scope.member_id
        ],
    )
    .map_err(|e| format!("更新指标失败: {}", e))?;

    Ok(Indicator {
        id: input.id,
        project_id: existing.project_id,
        name,
        unit,
        reference_range,
        sort_order,
        is_core,
        created_at: existing.created_at,
    })
}

fn delete_indicator_with_conn(
    conn: &Connection,
    id: String,
    scope: &ResolvedMemberScope,
) -> Result<bool, String> {
    // 检查是否有关联的指标值（仅当前成员）
    let value_count: i32 = conn
        .query_row(
            "SELECT COUNT(*)
             FROM indicator_values
             WHERE indicator_id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
            params![&id, &scope.owner_user_id, &scope.member_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    if value_count > 0 {
        return Err(format!("该指标有 {} 条历史数据，无法删除。", value_count));
    }

    let affected = conn
        .execute(
            "DELETE FROM indicators
             WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
            params![&id, &scope.owner_user_id, &scope.member_id],
        )
        .map_err(|e| format!("删除指标失败: {}", e))?;

    if affected == 0 {
        return Err("指标不存在或不属于当前成员".to_string());
    }

    Ok(true)
}

fn ensure_indicator_with_conn(
    conn: &Connection,
    input: CreateIndicatorInput,
    scope: &ResolvedMemberScope,
) -> Result<Indicator, String> {
    ensure_project_in_scope(conn, &input.project_id, scope)?;

    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM indicators
                WHERE project_id = ?1 AND owner_user_id = ?2 AND member_id = ?3 AND name = ?4
            )",
            params![
                &input.project_id,
                &scope.owner_user_id,
                &scope.member_id,
                &input.name
            ],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if exists {
        return Err("指标已存在".to_string());
    }

    let now = chrono::Local::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();
    let unit = input.unit.unwrap_or_default();
    let reference_range = input.reference_range.unwrap_or_default();
    let is_core = input.is_core.unwrap_or(true);

    conn.execute(
        "INSERT INTO indicators (
            id, project_id, owner_user_id, member_id, name, unit, reference_range, sort_order, is_core, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9)",
        params![
            id,
            input.project_id,
            &scope.owner_user_id,
            &scope.member_id,
            input.name,
            unit,
            reference_range,
            is_core as i32,
            now
        ],
    )
    .map_err(|e| format!("创建指标失败: {}", e))?;

    // 回填历史数据（仅当前成员+当前项目）
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, record_id, checkup_date, parsed_items
         FROM ocr_results
         WHERE project_id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
    ) {
        let ocr_rows = stmt.query_map(
            params![&input.project_id, &scope.owner_user_id, &scope.member_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        );

        if let Ok(rows) = ocr_rows {
            for row in rows {
                if let Ok((ocr_id, record_id, checkup_date, parsed_items_str)) = row {
                    if let Ok(items) =
                        serde_json::from_str::<Vec<serde_json::Value>>(&parsed_items_str)
                    {
                        for item in items {
                            let item_name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            if item_name.replace(" ", "") == input.name.replace(" ", "") {
                                let value_text = match item.get("value") {
                                    Some(serde_json::Value::String(s)) => s.clone(),
                                    Some(serde_json::Value::Number(n)) => n.to_string(),
                                    _ => String::new(),
                                };
                                let value: Option<f64> = value_text.parse().ok();
                                let is_abnormal = item
                                    .get("is_abnormal")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                let iv_id = uuid::Uuid::new_v4().to_string();

                                let _ = conn.execute(
                                    "INSERT INTO indicator_values (
                                        id, ocr_result_id, record_id, project_id, indicator_id,
                                        owner_user_id, member_id, checkup_date, value, value_text, is_abnormal, created_at
                                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                                    params![
                                        iv_id,
                                        ocr_id,
                                        record_id,
                                        input.project_id,
                                        id,
                                        &scope.owner_user_id,
                                        &scope.member_id,
                                        checkup_date,
                                        value,
                                        value_text,
                                        is_abnormal as i32,
                                        now
                                    ],
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(Indicator {
        id,
        project_id: input.project_id,
        name: input.name,
        unit,
        reference_range,
        sort_order: 0,
        is_core,
        created_at: now,
    })
}

#[tauri::command]
pub fn list_indicators(
    project_id: String,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<Vec<Indicator>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    list_indicators_with_conn(conn, project_id, &scope)
}

#[tauri::command]
pub fn create_indicator(
    input: CreateIndicatorInput,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<Indicator, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    create_indicator_with_conn(conn, input, &scope)
}

#[tauri::command]
pub fn update_indicator(
    input: UpdateIndicatorInput,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<Indicator, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    update_indicator_with_conn(conn, input, &scope)
}

#[tauri::command]
pub fn delete_indicator(
    id: String,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    delete_indicator_with_conn(conn, id, &scope)
}

#[tauri::command]
pub fn ensure_indicator(
    input: CreateIndicatorInput,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<Indicator, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    ensure_indicator_with_conn(conn, input, &scope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_database(name: &str) -> (Database, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "health-monitor-indicator-tests-{}-{}",
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

    fn seed_project(conn: &Connection, id: &str, owner_user_id: &str, member_id: &str) {
        conn.execute(
            "INSERT INTO checkup_projects (
                id, owner_user_id, member_id, name, description, sort_order, is_active, created_at, updated_at
             ) VALUES (?1, ?2, ?3, '血常规', '', 0, 1, '2026-04-08T00:00:00+08:00', '2026-04-08T00:00:00+08:00')",
            params![id, owner_user_id, member_id],
        )
        .expect("project should seed");
    }

    fn seed_record_and_ocr(
        conn: &Connection,
        owner_user_id: &str,
        member_id: &str,
        project_id: &str,
        record_id: &str,
        ocr_id: &str,
    ) {
        let file_id = format!("file-{ocr_id}");
        conn.execute(
            "INSERT INTO checkup_records (
                id, owner_user_id, member_id, member_name_snapshot, checkup_date, status, notes, created_at, updated_at
             ) VALUES (?1, ?2, ?3, '成员', '2026-04-08', 'ocr_done', '', '2026-04-08T00:00:00+08:00', '2026-04-08T00:00:00+08:00')",
            params![record_id, owner_user_id, member_id],
        )
        .expect("record should seed");
        conn.execute(
            "INSERT INTO checkup_files (
                id, owner_user_id, member_id, record_id, project_id, original_filename, stored_path, file_size, mime_type, uploaded_at
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, 'report.png', 'pictures/report.png', 100, 'image/png', '2026-04-08T00:00:00+08:00'
             )",
            params![&file_id, owner_user_id, member_id, record_id, project_id],
        )
        .expect("file should seed");
        conn.execute(
            "INSERT INTO ocr_results (
                id, owner_user_id, member_id, file_id, record_id, project_id, checkup_date, raw_json, parsed_items, status, error_message, created_at
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, '2026-04-08', '{}',
                '[{\"name\":\"白细胞\",\"value\":\"11.2\",\"unit\":\"10^9/L\",\"reference_range\":\"3.5-9.5\",\"is_abnormal\":true}]',
                'success', '', '2026-04-08T00:00:00+08:00'
             )",
            params![ocr_id, owner_user_id, member_id, &file_id, record_id, project_id],
        )
        .expect("ocr should seed");
    }

    #[test]
    fn list_indicators_only_returns_current_member_rows() {
        let (db, dir) = create_test_database("list");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");

            seed_project(conn, "proj-a", "user-1", "member-a");
            seed_project(conn, "proj-b", "user-1", "member-b");

            create_indicator_with_conn(
                conn,
                CreateIndicatorInput {
                    project_id: "proj-a".to_string(),
                    name: "白细胞".to_string(),
                    unit: Some("10^9/L".to_string()),
                    reference_range: Some("3.5-9.5".to_string()),
                    is_core: Some(true),
                },
                &member_scope("user-1", "member-a", "本人"),
            )
            .expect("member A indicator should create");

            create_indicator_with_conn(
                conn,
                CreateIndicatorInput {
                    project_id: "proj-b".to_string(),
                    name: "血红蛋白".to_string(),
                    unit: Some("g/L".to_string()),
                    reference_range: Some("120-160".to_string()),
                    is_core: Some(true),
                },
                &member_scope("user-1", "member-b", "母亲"),
            )
            .expect("member B indicator should create");

            let list_a = list_indicators_with_conn(
                conn,
                "proj-a".to_string(),
                &member_scope("user-1", "member-a", "本人"),
            )
            .expect("member A list should load");
            let list_b = list_indicators_with_conn(
                conn,
                "proj-b".to_string(),
                &member_scope("user-1", "member-b", "母亲"),
            )
            .expect("member B list should load");

            assert_eq!(list_a.len(), 1);
            assert_eq!(list_b.len(), 1);
            assert_eq!(list_a[0].name, "白细胞");
            assert_eq!(list_b[0].name, "血红蛋白");
        }
        cleanup_test_database(&db, dir);
    }

    #[test]
    fn ensure_indicator_only_backfills_current_member_ocr_rows() {
        let (db, dir) = create_test_database("ensure-backfill");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");

            seed_project(conn, "proj-a", "user-1", "member-a");
            seed_project(conn, "proj-b", "user-1", "member-b");
            seed_record_and_ocr(conn, "user-1", "member-a", "proj-a", "record-a", "ocr-a");
            seed_record_and_ocr(conn, "user-1", "member-b", "proj-b", "record-b", "ocr-b");

            let created = ensure_indicator_with_conn(
                conn,
                CreateIndicatorInput {
                    project_id: "proj-a".to_string(),
                    name: "白细胞".to_string(),
                    unit: Some("10^9/L".to_string()),
                    reference_range: Some("3.5-9.5".to_string()),
                    is_core: Some(true),
                },
                &member_scope("user-1", "member-a", "本人"),
            )
            .expect("ensure indicator should succeed");

            let member_a_values: i64 = conn
                .query_row(
                    "SELECT COUNT(*)
                     FROM indicator_values
                     WHERE owner_user_id = 'user-1' AND member_id = 'member-a' AND indicator_id = ?1",
                    params![created.id],
                    |row| row.get(0),
                )
                .expect("count should succeed");
            let member_b_values: i64 = conn
                .query_row(
                    "SELECT COUNT(*)
                     FROM indicator_values
                     WHERE owner_user_id = 'user-1' AND member_id = 'member-b' AND indicator_id = ?1",
                    params![created.id],
                    |row| row.get(0),
                )
                .expect("count should succeed");

            assert_eq!(member_a_values, 1);
            assert_eq!(member_b_values, 0);
        }
        cleanup_test_database(&db, dir);
    }
}
