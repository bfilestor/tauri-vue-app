use crate::db::Database;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FamilyMember {
    pub member_id: String,
    pub member_name: String,
    pub relation_code: String,
    pub gender: String,
    pub birthday: String,
    pub mobile: String,
    pub health_note: String,
    pub is_default: bool,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFamilyMemberInput {
    pub member_name: String,
    pub relation_code: String,
    pub gender: Option<String>,
    pub birthday: Option<String>,
    pub mobile: Option<String>,
    pub health_note: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFamilyMemberInput {
    pub member_name: Option<String>,
    pub relation_code: Option<String>,
    pub gender: Option<String>,
    pub birthday: Option<String>,
    pub mobile: Option<String>,
    pub health_note: Option<String>,
}

fn normalize_text(value: Option<String>) -> String {
    value.unwrap_or_default().trim().to_string()
}

fn normalize_owner_user_id(owner_user_id: String) -> Result<String, String> {
    let normalized = owner_user_id.trim().to_string();
    if normalized.is_empty() {
        return Err("用户未登录，请先登录后再管理家庭成员".to_string());
    }

    Ok(normalized)
}

fn normalize_required_text(value: String, field_name: &str) -> Result<String, String> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        return Err(format!("{field_name}不能为空"));
    }

    Ok(normalized)
}

fn normalize_optional_relation_code(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_uppercase())
        .and_then(|item| if item.is_empty() { None } else { Some(item) })
}

fn list_family_members_with_conn(
    conn: &Connection,
    owner_user_id: &str,
) -> Result<Vec<FamilyMember>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT cloud_member_id, member_name, relation_code, gender, birthday, mobile, health_note, is_default, status, created_at, updated_at
             FROM family_members
             WHERE owner_user_id = ?1 AND status = 'ENABLED'
             ORDER BY is_default DESC, created_at ASC",
        )
        .map_err(|e| format!("查询家庭成员失败: {}", e))?;

    let list = stmt
        .query_map([owner_user_id], |row| {
            Ok(FamilyMember {
                member_id: row.get(0)?,
                member_name: row.get(1)?,
                relation_code: row.get(2)?,
                gender: row.get(3)?,
                birthday: row.get(4)?,
                mobile: row.get(5)?,
                health_note: row.get(6)?,
                is_default: row.get::<_, i32>(7)? == 1,
                status: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| format!("查询家庭成员失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析家庭成员失败: {}", e))?;

    Ok(list)
}

fn create_family_member_with_conn(
    conn: &Connection,
    owner_user_id: &str,
    input: CreateFamilyMemberInput,
) -> Result<FamilyMember, String> {
    let member_name = normalize_required_text(input.member_name, "成员姓名")?;
    let relation_code = normalize_required_text(
        normalize_optional_relation_code(Some(input.relation_code))
            .unwrap_or_else(|| "OTHER".to_string()),
        "关系",
    )?;

    let member_id = uuid::Uuid::new_v4().to_string();
    let row_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().to_rfc3339();
    let gender = normalize_text(input.gender);
    let birthday = normalize_text(input.birthday);
    let mobile = normalize_text(input.mobile);
    let health_note = normalize_text(input.health_note);

    let enabled_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM family_members WHERE owner_user_id = ?1 AND status = 'ENABLED'",
            [owner_user_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询成员数量失败: {}", e))?;

    let should_set_default = input.is_default.unwrap_or(false) || enabled_count == 0;
    if should_set_default {
        conn.execute(
            "UPDATE family_members SET is_default = 0, updated_at = ?1 WHERE owner_user_id = ?2",
            params![now, owner_user_id],
        )
        .map_err(|e| format!("重置默认成员失败: {}", e))?;
    }

    conn.execute(
        "INSERT INTO family_members (
            id, cloud_member_id, owner_user_id, member_name, relation_code, gender, birthday, mobile, health_note,
            is_default, status, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'ENABLED', ?11, ?11)",
        params![
            row_id,
            member_id,
            owner_user_id,
            member_name,
            relation_code,
            gender,
            birthday,
            mobile,
            health_note,
            if should_set_default { 1 } else { 0 },
            now
        ],
    )
    .map_err(|e| format!("新增成员失败: {}", e))?;

    Ok(FamilyMember {
        member_id,
        member_name,
        relation_code,
        gender,
        birthday,
        mobile,
        health_note,
        is_default: should_set_default,
        status: "ENABLED".to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

fn update_family_member_with_conn(
    conn: &Connection,
    owner_user_id: &str,
    member_id: &str,
    input: UpdateFamilyMemberInput,
) -> Result<bool, String> {
    let existing = conn
        .query_row(
            "SELECT member_name, relation_code, gender, birthday, mobile, health_note
             FROM family_members
             WHERE owner_user_id = ?1 AND cloud_member_id = ?2 AND status = 'ENABLED'",
            params![owner_user_id, member_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .optional()
        .map_err(|e| format!("查询成员失败: {}", e))?
        .ok_or("成员不存在或已失效".to_string())?;

    let next_name = match input.member_name {
        Some(value) => normalize_required_text(value, "成员姓名")?,
        None => existing.0,
    };
    let next_relation = match input.relation_code {
        Some(value) => normalize_required_text(
            normalize_optional_relation_code(Some(value)).unwrap_or_else(|| "OTHER".to_string()),
            "关系",
        )?,
        None => existing.1,
    };
    let next_gender = input.gender.map(|item| item.trim().to_string()).unwrap_or(existing.2);
    let next_birthday = input
        .birthday
        .map(|item| item.trim().to_string())
        .unwrap_or(existing.3);
    let next_mobile = input.mobile.map(|item| item.trim().to_string()).unwrap_or(existing.4);
    let next_health_note = input
        .health_note
        .map(|item| item.trim().to_string())
        .unwrap_or(existing.5);
    let now = chrono::Local::now().to_rfc3339();

    conn.execute(
        "UPDATE family_members
         SET member_name = ?1,
             relation_code = ?2,
             gender = ?3,
             birthday = ?4,
             mobile = ?5,
             health_note = ?6,
             updated_at = ?7
         WHERE owner_user_id = ?8 AND cloud_member_id = ?9 AND status = 'ENABLED'",
        params![
            next_name,
            next_relation,
            next_gender,
            next_birthday,
            next_mobile,
            next_health_note,
            now,
            owner_user_id,
            member_id
        ],
    )
    .map_err(|e| format!("更新成员失败: {}", e))?;

    Ok(true)
}

fn set_default_family_member_with_conn(
    conn: &Connection,
    owner_user_id: &str,
    member_id: &str,
) -> Result<bool, String> {
    let exists = conn
        .query_row(
            "SELECT COUNT(*) FROM family_members
             WHERE owner_user_id = ?1 AND cloud_member_id = ?2 AND status = 'ENABLED'",
            params![owner_user_id, member_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| format!("查询成员失败: {}", e))?;

    if exists == 0 {
        return Err("成员不存在或已失效".to_string());
    }

    let now = chrono::Local::now().to_rfc3339();
    conn.execute(
        "UPDATE family_members
         SET is_default = CASE WHEN cloud_member_id = ?1 THEN 1 ELSE 0 END,
             updated_at = ?2
         WHERE owner_user_id = ?3 AND status = 'ENABLED'",
        params![member_id, now, owner_user_id],
    )
    .map_err(|e| format!("设置默认成员失败: {}", e))?;

    Ok(true)
}

fn delete_family_member_with_conn(
    conn: &Connection,
    owner_user_id: &str,
    member_id: &str,
) -> Result<bool, String> {
    let target = conn
        .query_row(
            "SELECT is_default FROM family_members
             WHERE owner_user_id = ?1 AND cloud_member_id = ?2 AND status = 'ENABLED'",
            params![owner_user_id, member_id],
            |row| row.get::<_, i32>(0),
        )
        .optional()
        .map_err(|e| format!("查询成员失败: {}", e))?;

    let target_is_default = match target {
        Some(value) => value == 1,
        None => return Err("成员不存在或已失效".to_string()),
    };

    let enabled_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM family_members WHERE owner_user_id = ?1 AND status = 'ENABLED'",
            [owner_user_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询成员数量失败: {}", e))?;

    if enabled_count <= 1 {
        return Err("至少保留 1 个家庭成员".to_string());
    }

    conn.execute(
        "DELETE FROM family_members
         WHERE owner_user_id = ?1 AND cloud_member_id = ?2 AND status = 'ENABLED'",
        params![owner_user_id, member_id],
    )
    .map_err(|e| format!("删除成员失败: {}", e))?;

    if target_is_default {
        let fallback_member_id = conn
            .query_row(
                "SELECT cloud_member_id FROM family_members
                 WHERE owner_user_id = ?1 AND status = 'ENABLED'
                 ORDER BY created_at ASC
                 LIMIT 1",
                [owner_user_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| format!("查询候选默认成员失败: {}", e))?
            .ok_or("删除成员后未找到可用成员".to_string())?;

        set_default_family_member_with_conn(conn, owner_user_id, &fallback_member_id)?;
    }

    Ok(true)
}

#[tauri::command]
pub fn list_family_members(
    owner_user_id: String,
    db: State<Database>,
) -> Result<Vec<FamilyMember>, String> {
    let owner_user_id = normalize_owner_user_id(owner_user_id)?;
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    list_family_members_with_conn(conn, &owner_user_id)
}

#[tauri::command]
pub fn create_family_member(
    owner_user_id: String,
    input: CreateFamilyMemberInput,
    db: State<Database>,
) -> Result<FamilyMember, String> {
    let owner_user_id = normalize_owner_user_id(owner_user_id)?;
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    create_family_member_with_conn(conn, &owner_user_id, input)
}

#[tauri::command]
pub fn update_family_member(
    owner_user_id: String,
    member_id: String,
    input: UpdateFamilyMemberInput,
    db: State<Database>,
) -> Result<bool, String> {
    let owner_user_id = normalize_owner_user_id(owner_user_id)?;
    let member_id = normalize_required_text(member_id, "成员ID")?;
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    update_family_member_with_conn(conn, &owner_user_id, &member_id, input)
}

#[tauri::command]
pub fn delete_family_member(
    owner_user_id: String,
    member_id: String,
    db: State<Database>,
) -> Result<bool, String> {
    let owner_user_id = normalize_owner_user_id(owner_user_id)?;
    let member_id = normalize_required_text(member_id, "成员ID")?;
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    delete_family_member_with_conn(conn, &owner_user_id, &member_id)
}

#[tauri::command]
pub fn set_default_family_member(
    owner_user_id: String,
    member_id: String,
    db: State<Database>,
) -> Result<bool, String> {
    let owner_user_id = normalize_owner_user_id(owner_user_id)?;
    let member_id = normalize_required_text(member_id, "成员ID")?;
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    set_default_family_member_with_conn(conn, &owner_user_id, &member_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_database(name: &str) -> (Database, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "health-monitor-member-tests-{}-{}",
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

    fn create_member(
        conn: &Connection,
        owner_user_id: &str,
        member_name: &str,
        relation_code: &str,
        is_default: Option<bool>,
    ) -> FamilyMember {
        create_family_member_with_conn(
            conn,
            owner_user_id,
            CreateFamilyMemberInput {
                member_name: member_name.to_string(),
                relation_code: relation_code.to_string(),
                gender: None,
                birthday: None,
                mobile: None,
                health_note: None,
                is_default,
            },
        )
        .expect("create member should succeed")
    }

    #[test]
    fn create_family_member_marks_first_member_as_default() {
        let (db, dir) = create_test_database("create-default");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");

            let created = create_member(conn, "user-1", "本人", "SELF", None);
            assert!(created.is_default);

            let list = list_family_members_with_conn(conn, "user-1").expect("list should succeed");
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].member_name, "本人");
            assert!(list[0].is_default);
        }
        cleanup_test_database(&db, dir);
    }

    #[test]
    fn set_default_family_member_only_affects_current_owner() {
        let (db, dir) = create_test_database("set-default-owner");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");

            let user1_a = create_member(conn, "user-1", "本人", "SELF", None);
            let user1_b = create_member(conn, "user-1", "母亲", "MOTHER", None);
            let user2_a = create_member(conn, "user-2", "本人", "SELF", None);

            set_default_family_member_with_conn(conn, "user-1", &user1_b.member_id)
                .expect("set default should succeed");

            let user1_list =
                list_family_members_with_conn(conn, "user-1").expect("user-1 list should succeed");
            let user2_list =
                list_family_members_with_conn(conn, "user-2").expect("user-2 list should succeed");

            let user1_default = user1_list
                .iter()
                .find(|item| item.is_default)
                .expect("user-1 default should exist");
            assert_eq!(user1_default.member_id, user1_b.member_id);
            assert_ne!(user1_default.member_id, user1_a.member_id);

            let user2_default = user2_list
                .iter()
                .find(|item| item.is_default)
                .expect("user-2 default should exist");
            assert_eq!(user2_default.member_id, user2_a.member_id);
        }
        cleanup_test_database(&db, dir);
    }

    #[test]
    fn delete_family_member_reassigns_default_and_blocks_last_member_delete() {
        let (db, dir) = create_test_database("delete-member");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");

            let member_a = create_member(conn, "user-1", "本人", "SELF", None);
            let member_b = create_member(conn, "user-1", "父亲", "FATHER", None);

            delete_family_member_with_conn(conn, "user-1", &member_a.member_id)
                .expect("delete default member should succeed");

            let list_after_delete =
                list_family_members_with_conn(conn, "user-1").expect("list should succeed");
            assert_eq!(list_after_delete.len(), 1);
            assert_eq!(list_after_delete[0].member_id, member_b.member_id);
            assert!(list_after_delete[0].is_default);

            let delete_last_err = delete_family_member_with_conn(conn, "user-1", &member_b.member_id)
                .expect_err("delete last member should fail");
            assert!(delete_last_err.contains("至少保留 1 个家庭成员"));
        }
        cleanup_test_database(&db, dir);
    }

    #[test]
    fn list_family_members_isolated_by_owner_user_id() {
        let (db, dir) = create_test_database("owner-isolation");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");

            create_member(conn, "user-1", "本人", "SELF", None);
            create_member(conn, "user-1", "母亲", "MOTHER", None);
            create_member(conn, "user-2", "本人", "SELF", None);

            let user1_list =
                list_family_members_with_conn(conn, "user-1").expect("user-1 list should succeed");
            let user2_list =
                list_family_members_with_conn(conn, "user-2").expect("user-2 list should succeed");

            assert_eq!(user1_list.len(), 2);
            assert_eq!(user2_list.len(), 1);
            assert_eq!(user2_list[0].member_name, "本人");
        }
        cleanup_test_database(&db, dir);
    }
}
