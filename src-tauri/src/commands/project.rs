use super::AppDir;
use super::scope::{MemberScopeInput, ResolvedMemberScope, resolve_member_scope};
use crate::db::Database;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectInput {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectInput {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

fn list_projects_with_conn(
    conn: &rusqlite::Connection,
    scope: &ResolvedMemberScope,
) -> Result<Vec<Project>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, sort_order, is_active, created_at, updated_at
             FROM checkup_projects
             WHERE owner_user_id = ?1 AND member_id = ?2
             ORDER BY sort_order ASC, created_at ASC",
        )
        .map_err(|e| format!("查询项目失败: {}", e))?;

    let projects = stmt
        .query_map(
            rusqlite::params![&scope.owner_user_id, &scope.member_id],
            |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    sort_order: row.get(3)?,
                    is_active: row.get::<_, i32>(4)? == 1,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
        .map_err(|e| format!("查询项目失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析项目数据失败: {}", e))?;

    Ok(projects)
}

fn create_project_with_conn(
    conn: &rusqlite::Connection,
    input: CreateProjectInput,
    scope: &ResolvedMemberScope,
    app_dir: &Path,
) -> Result<Project, String> {
    let now = chrono::Local::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();
    let description = input.description.unwrap_or_default();

    // 创建对应的 pictures 子目录（成员级）
    let project_dir = app_dir
        .join("pictures")
        .join(&scope.owner_user_id)
        .join(&scope.member_id)
        .join(&input.name);
    std::fs::create_dir_all(&project_dir).map_err(|e| format!("创建项目文件夹失败: {}", e))?;

    conn.execute(
        "INSERT INTO checkup_projects (
            id, owner_user_id, member_id, name, description, sort_order, is_active, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, 0, 1, ?6, ?7)",
        rusqlite::params![
            id,
            &scope.owner_user_id,
            &scope.member_id,
            input.name,
            description,
            now,
            now
        ],
    )
    .map_err(|e| format!("创建项目失败: {}", e))?;

    Ok(Project {
        id,
        name: input.name,
        description,
        sort_order: 0,
        is_active: true,
        created_at: now.clone(),
        updated_at: now,
    })
}

fn update_project_with_conn(
    conn: &rusqlite::Connection,
    input: UpdateProjectInput,
    scope: &ResolvedMemberScope,
) -> Result<Project, String> {
    let now = chrono::Local::now().to_rfc3339();

    let existing = conn
        .query_row(
            "SELECT id, name, description, sort_order, is_active, created_at
             FROM checkup_projects
             WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
            rusqlite::params![&input.id, &scope.owner_user_id, &scope.member_id],
            |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    sort_order: row.get(3)?,
                    is_active: row.get::<_, i32>(4)? == 1,
                    created_at: row.get(5)?,
                    updated_at: String::new(),
                })
            },
        )
        .map_err(|e| format!("项目不存在或不属于当前成员: {}", e))?;

    let name = input.name.unwrap_or(existing.name);
    let description = input.description.unwrap_or(existing.description);
    let is_active = input.is_active.unwrap_or(existing.is_active);
    let sort_order = input.sort_order.unwrap_or(existing.sort_order);

    conn.execute(
        "UPDATE checkup_projects
         SET name = ?1, description = ?2, is_active = ?3, sort_order = ?4, updated_at = ?5
         WHERE id = ?6 AND owner_user_id = ?7 AND member_id = ?8",
        rusqlite::params![
            name,
            description,
            is_active as i32,
            sort_order,
            now,
            input.id,
            &scope.owner_user_id,
            &scope.member_id
        ],
    )
    .map_err(|e| format!("更新项目失败: {}", e))?;

    Ok(Project {
        id: input.id,
        name,
        description,
        sort_order,
        is_active,
        created_at: existing.created_at,
        updated_at: now,
    })
}

fn delete_project_with_conn(
    conn: &rusqlite::Connection,
    id: String,
    scope: &ResolvedMemberScope,
) -> Result<bool, String> {
    let project_exists: bool = conn
        .query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM checkup_projects
                WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3
            )",
            rusqlite::params![&id, &scope.owner_user_id, &scope.member_id],
            |row| row.get::<_, i32>(0),
        )
        .map_err(|e| format!("查询项目失败: {}", e))?
        == 1;

    if !project_exists {
        return Err("项目不存在或不属于当前成员".to_string());
    }

    // 仅检查当前成员的关联文件
    let file_count: i32 = conn
        .query_row(
            "SELECT COUNT(*)
             FROM checkup_files
             WHERE project_id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
            rusqlite::params![&id, &scope.owner_user_id, &scope.member_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询关联文件失败: {}", e))?;

    if file_count > 0 {
        return Err(format!(
            "该项目下有 {} 个关联文件，无法删除。请先删除相关检查记录。",
            file_count
        ));
    }

    conn.execute(
        "DELETE FROM indicators
         WHERE project_id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
        rusqlite::params![&id, &scope.owner_user_id, &scope.member_id],
    )
    .map_err(|e| format!("删除指标失败: {}", e))?;

    conn.execute(
        "DELETE FROM checkup_projects
         WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
        rusqlite::params![&id, &scope.owner_user_id, &scope.member_id],
    )
    .map_err(|e| format!("删除项目失败: {}", e))?;

    Ok(true)
}

#[tauri::command]
pub fn list_projects(
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<Vec<Project>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    list_projects_with_conn(conn, &scope)
}

#[tauri::command]
pub fn create_project(
    input: CreateProjectInput,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
    app_dir: State<AppDir>,
) -> Result<Project, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    create_project_with_conn(conn, input, &scope, &app_dir.0)
}

#[tauri::command]
pub fn update_project(
    input: UpdateProjectInput,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<Project, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    update_project_with_conn(conn, input, &scope)
}

#[tauri::command]
pub fn delete_project(
    id: String,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    delete_project_with_conn(conn, id, &scope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_database(name: &str) -> (Database, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "health-monitor-project-tests-{}-{}",
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

    fn seed_record(
        conn: &rusqlite::Connection,
        owner_user_id: &str,
        member_id: &str,
        record_id: &str,
    ) {
        conn.execute(
            "INSERT INTO checkup_records (
                id, owner_user_id, member_id, member_name_snapshot, checkup_date, status, notes, created_at, updated_at
             ) VALUES (?1, ?2, ?3, '成员', '2026-04-08', 'pending_upload', '', '2026-04-08T00:00:00+08:00', '2026-04-08T00:00:00+08:00')",
            rusqlite::params![record_id, owner_user_id, member_id],
        )
        .expect("record should seed");
    }

    #[test]
    fn list_projects_only_returns_current_member_rows() {
        let (db, dir) = create_test_database("list");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");
            create_project_with_conn(
                conn,
                CreateProjectInput {
                    name: "血常规".to_string(),
                    description: Some("A".to_string()),
                },
                &member_scope("user-1", "member-a", "本人"),
                &dir,
            )
            .expect("member A project should create");
            create_project_with_conn(
                conn,
                CreateProjectInput {
                    name: "血常规".to_string(),
                    description: Some("B".to_string()),
                },
                &member_scope("user-1", "member-b", "母亲"),
                &dir,
            )
            .expect("member B project should create");

            let list_a = list_projects_with_conn(conn, &member_scope("user-1", "member-a", "本人"))
                .expect("member A list should succeed");
            let list_b = list_projects_with_conn(conn, &member_scope("user-1", "member-b", "母亲"))
                .expect("member B list should succeed");

            assert_eq!(list_a.len(), 1);
            assert_eq!(list_b.len(), 1);
            assert_eq!(list_a[0].description, "A");
            assert_eq!(list_b[0].description, "B");
        }
        cleanup_test_database(&db, dir);
    }

    #[test]
    fn update_project_rejects_cross_member_access() {
        let (db, dir) = create_test_database("update");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");
            let project = create_project_with_conn(
                conn,
                CreateProjectInput {
                    name: "血脂".to_string(),
                    description: None,
                },
                &member_scope("user-1", "member-a", "本人"),
                &dir,
            )
            .expect("project should create");

            let err = update_project_with_conn(
                conn,
                UpdateProjectInput {
                    id: project.id,
                    name: Some("肝功".to_string()),
                    description: None,
                    is_active: None,
                    sort_order: None,
                },
                &member_scope("user-1", "member-b", "母亲"),
            )
            .expect_err("cross-member update should fail");

            assert!(err.contains("不属于当前成员"));
        }
        cleanup_test_database(&db, dir);
    }

    #[test]
    fn delete_project_only_checks_current_member_file_binding() {
        let (db, dir) = create_test_database("delete");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");
            let scope_a = member_scope("user-1", "member-a", "本人");
            let scope_b = member_scope("user-1", "member-b", "母亲");

            let project_a = create_project_with_conn(
                conn,
                CreateProjectInput {
                    name: "A项目".to_string(),
                    description: None,
                },
                &scope_a,
                &dir,
            )
            .expect("project A should create");
            let project_b = create_project_with_conn(
                conn,
                CreateProjectInput {
                    name: "B项目".to_string(),
                    description: None,
                },
                &scope_b,
                &dir,
            )
            .expect("project B should create");

            seed_record(conn, "user-1", "member-b", "record-b");
            conn.execute(
                "INSERT INTO checkup_files (
                    id, owner_user_id, member_id, record_id, project_id, original_filename, stored_path, file_size, mime_type, uploaded_at
                 ) VALUES (
                    'file-b', 'user-1', 'member-b', 'record-b', ?1, 'report.png', 'pictures/report.png', 10, 'image/png', '2026-04-08T00:00:00+08:00'
                 )",
                rusqlite::params![&project_b.id],
            )
            .expect("member B file should seed");

            let deleted_a = delete_project_with_conn(conn, project_a.id.clone(), &scope_a)
                .expect("member A delete should succeed");
            assert!(deleted_a);

            let cross_delete_err = delete_project_with_conn(conn, project_b.id.clone(), &scope_a)
                .expect_err("cross-member delete should fail");
            assert!(cross_delete_err.contains("不属于当前成员"));

            let own_delete_err = delete_project_with_conn(conn, project_b.id, &scope_b)
                .expect_err("project with own files should fail");
            assert!(own_delete_err.contains("关联文件"));
        }
        cleanup_test_database(&db, dir);
    }
}
