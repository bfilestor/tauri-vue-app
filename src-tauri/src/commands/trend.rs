use super::scope::{MemberScopeInput, ResolvedMemberScope, resolve_member_scope};
use crate::db::Database;
use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct TrendDataPoint {
    pub checkup_date: String,
    pub value: Option<f64>,
    pub value_text: String,
    pub is_abnormal: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct IndicatorTrend {
    pub indicator_id: String,
    pub indicator_name: String,
    pub unit: String,
    pub reference_range: String,
    pub data_points: Vec<TrendDataPoint>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProjectTrend {
    pub project_id: String,
    pub project_name: String,
    pub indicators: Vec<IndicatorTrend>,
}

/// 获取某个项目的趋势数据
fn get_project_trends_with_conn(
    conn: &Connection,
    project_id: String,
    scope: &ResolvedMemberScope,
) -> Result<ProjectTrend, String> {
    // 获取项目名称
    let project_name: String = conn
        .query_row(
            "SELECT name
             FROM checkup_projects
             WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
            rusqlite::params![&project_id, &scope.owner_user_id, &scope.member_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("项目不存在: {}", e))?;

    // 获取该项目的所有指标
    let mut ind_stmt = conn
        .prepare(
            "SELECT id, name, unit, reference_range FROM indicators
             WHERE project_id = ?1 AND owner_user_id = ?2 AND member_id = ?3
             ORDER BY is_core DESC, sort_order ASC, name ASC",
        )
        .map_err(|e| format!("查询指标失败: {}", e))?;

    let indicators: Vec<(String, String, String, String)> = ind_stmt
        .query_map(
            rusqlite::params![&project_id, &scope.owner_user_id, &scope.member_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2).unwrap_or_default(),
                    row.get::<_, String>(3).unwrap_or_default(),
                ))
            },
        )
        .map_err(|e| format!("查询指标失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析指标数据失败: {}", e))?;

    // 获取每个指标的历史值
    let mut trend_indicators = Vec::new();

    for (ind_id, ind_name, unit, ref_range) in &indicators {
        let mut val_stmt = conn
            .prepare(
                "SELECT checkup_date, value, value_text, is_abnormal
                 FROM indicator_values
                 WHERE indicator_id = ?1
                   AND project_id = ?2
                   AND owner_user_id = ?3
                   AND member_id = ?4
                 ORDER BY checkup_date ASC",
            )
            .map_err(|e| format!("查询指标值失败: {}", e))?;

        let data_points: Vec<TrendDataPoint> = val_stmt
            .query_map(
                rusqlite::params![ind_id, project_id, &scope.owner_user_id, &scope.member_id],
                |row| {
                    Ok(TrendDataPoint {
                        checkup_date: row.get(0)?,
                        value: row.get(1)?,
                        value_text: row.get::<_, String>(2).unwrap_or_default(),
                        is_abnormal: row.get::<_, i32>(3).unwrap_or(0) != 0,
                    })
                },
            )
            .map_err(|e| format!("查询失败: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("解析数据失败: {}", e))?;

        trend_indicators.push(IndicatorTrend {
            indicator_id: ind_id.clone(),
            indicator_name: ind_name.clone(),
            unit: unit.clone(),
            reference_range: ref_range.clone(),
            data_points,
        });
    }

    Ok(ProjectTrend {
        project_id,
        project_name,
        indicators: trend_indicators,
    })
}

/// 获取某个项目的趋势数据
#[tauri::command]
pub fn get_project_trends(
    project_id: String,
    scope: Option<MemberScopeInput>,
    db: tauri::State<Database>,
) -> Result<ProjectTrend, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    get_project_trends_with_conn(conn, project_id, &scope)
}

/// 获取所有项目的概要趋势数据
#[tauri::command]
pub fn get_all_trends(
    scope: Option<MemberScopeInput>,
    db: tauri::State<Database>,
) -> Result<Vec<ProjectTrend>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;

    // 获取所有活跃项目
    let mut proj_stmt = conn
        .prepare(
            "SELECT id, name
             FROM checkup_projects
             WHERE is_active = 1 AND owner_user_id = ?1 AND member_id = ?2
             ORDER BY sort_order ASC",
        )
        .map_err(|e| format!("查询项目失败: {}", e))?;

    let projects: Vec<(String, String)> = proj_stmt
        .query_map(
            rusqlite::params![&scope.owner_user_id, &scope.member_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|e| format!("查询项目失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析项目数据失败: {}", e))?;

    drop(proj_stmt);
    drop(conn_guard);

    let mut result = Vec::new();
    for (pid, _pname) in &projects {
        match get_project_trends(
            pid.clone(),
            Some(MemberScopeInput {
                owner_user_id: Some(scope.owner_user_id.clone()),
                member_id: Some(scope.member_id.clone()),
                member_name: Some(scope.member_name.clone()),
            }),
            db.clone(),
        ) {
            Ok(pt) => result.push(pt),
            Err(e) => log::error!("获取项目趋势失败: {}", e),
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_database(name: &str) -> (Database, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "health-monitor-trend-tests-{}-{}",
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

    #[test]
    fn get_project_trends_only_returns_current_member_indicator_values() {
        let (db, dir) = create_test_database("project-trends");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");

            conn.execute(
                "INSERT INTO checkup_projects (
                    id, owner_user_id, member_id, name, description, sort_order, is_active, created_at, updated_at
                 ) VALUES ('proj-blood', 'user-1', 'member-a', '血常规', '', 0, 1, '2026-04-08T00:00:00+08:00', '2026-04-08T00:00:00+08:00')",
                [],
            )
            .expect("project should seed");
            conn.execute(
                "INSERT INTO indicators (
                    id, project_id, owner_user_id, member_id, name, unit, reference_range, sort_order, is_core, created_at
                 ) VALUES ('ind-wbc', 'proj-blood', 'user-1', 'member-a', '白细胞', '10^9/L', '3.5-9.5', 0, 1, '2026-04-08T00:00:00+08:00')",
                [],
            )
            .expect("indicator should seed");
            conn.execute(
                "INSERT INTO checkup_records (
                    id, owner_user_id, member_id, member_name_snapshot, checkup_date, status, notes, created_at, updated_at
                 ) VALUES
                    ('record-a1', 'user-1', 'member-a', '成员A', '2026-04-01', 'ocr_done', '', '2026-04-01T00:00:00+08:00', '2026-04-01T00:00:00+08:00'),
                    ('record-a2', 'user-1', 'member-a', '成员A', '2026-04-08', 'ocr_done', '', '2026-04-08T00:00:00+08:00', '2026-04-08T00:00:00+08:00'),
                    ('record-b1', 'user-1', 'member-b', '成员B', '2026-04-08', 'ocr_done', '', '2026-04-08T00:00:00+08:00', '2026-04-08T00:00:00+08:00')",
                [],
            )
            .expect("records should seed");
            conn.execute(
                "INSERT INTO checkup_files (
                    id, owner_user_id, member_id, record_id, project_id, original_filename, stored_path, file_size, mime_type, uploaded_at
                 ) VALUES
                    ('file-a1', 'user-1', 'member-a', 'record-a1', 'proj-blood', 'a1.png', 'pictures/a1.png', 100, 'image/png', '2026-04-01T00:00:00+08:00'),
                    ('file-a2', 'user-1', 'member-a', 'record-a2', 'proj-blood', 'a2.png', 'pictures/a2.png', 100, 'image/png', '2026-04-08T00:00:00+08:00'),
                    ('file-b1', 'user-1', 'member-b', 'record-b1', 'proj-blood', 'b1.png', 'pictures/b1.png', 100, 'image/png', '2026-04-08T00:00:00+08:00')",
                [],
            )
            .expect("files should seed");
            conn.execute(
                "INSERT INTO ocr_results (
                    id, file_id, record_id, project_id, owner_user_id, member_id, checkup_date, raw_json, parsed_items, status, error_message, created_at
                 ) VALUES
                    ('ocr-a1', 'file-a1', 'record-a1', 'proj-blood', 'user-1', 'member-a', '2026-04-01', '{}', '[]', 'success', '', '2026-04-01T00:00:00+08:00'),
                    ('ocr-a2', 'file-a2', 'record-a2', 'proj-blood', 'user-1', 'member-a', '2026-04-08', '{}', '[]', 'success', '', '2026-04-08T00:00:00+08:00'),
                    ('ocr-b1', 'file-b1', 'record-b1', 'proj-blood', 'user-1', 'member-b', '2026-04-08', '{}', '[]', 'success', '', '2026-04-08T00:00:00+08:00')",
                [],
            )
            .expect("ocr rows should seed");
            conn.execute(
                "INSERT INTO indicator_values (
                    id, ocr_result_id, record_id, project_id, indicator_id, owner_user_id, member_id, checkup_date, value, value_text, is_abnormal, created_at
                 ) VALUES
                    ('iv-a1', 'ocr-a1', 'record-a1', 'proj-blood', 'ind-wbc', 'user-1', 'member-a', '2026-04-01', 11.2, '11.2', 1, '2026-04-01T00:00:00+08:00'),
                    ('iv-a2', 'ocr-a2', 'record-a2', 'proj-blood', 'ind-wbc', 'user-1', 'member-a', '2026-04-08', 8.3, '8.3', 0, '2026-04-08T00:00:00+08:00'),
                    ('iv-b1', 'ocr-b1', 'record-b1', 'proj-blood', 'ind-wbc', 'user-1', 'member-b', '2026-04-08', 15.0, '15.0', 1, '2026-04-08T00:00:00+08:00')",
                [],
            )
            .expect("indicator values should seed");

            let trend = get_project_trends_with_conn(
                conn,
                "proj-blood".to_string(),
                &member_scope("user-1", "member-a", "本人"),
            )
            .expect("trend should load");

            assert_eq!(trend.project_id, "proj-blood");
            assert_eq!(trend.indicators.len(), 1);
            assert_eq!(trend.indicators[0].indicator_id, "ind-wbc");
            assert_eq!(trend.indicators[0].data_points.len(), 2);
            assert_eq!(
                trend.indicators[0].data_points[0].checkup_date,
                "2026-04-01"
            );
            assert_eq!(
                trend.indicators[0].data_points[1].checkup_date,
                "2026-04-08"
            );
            assert!(
                trend.indicators[0]
                    .data_points
                    .iter()
                    .all(|point| point.value_text != "15.0")
            );
        }
        cleanup_test_database(&db, dir);
    }
}
