use super::scope::{MemberScopeInput, resolve_member_scope};
use crate::db::Database;
use serde::{Deserialize, Serialize};
use tauri::State;

const MEMBER_SCOPED_CONFIG_KEYS: &[&str] = &["user_custom_prompt_template"];

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigValue {
    pub config_key: String,
    pub config_value: String,
}

fn requires_member_scope(key: &str) -> bool {
    MEMBER_SCOPED_CONFIG_KEYS.iter().any(|item| *item == key)
}

fn build_member_scoped_key(owner_user_id: &str, member_id: &str, key: &str) -> String {
    format!("member:{owner_user_id}:{member_id}:{key}")
}

fn resolve_effective_key(
    conn: &rusqlite::Connection,
    key: &str,
    scope: Option<MemberScopeInput>,
) -> Result<String, String> {
    if !requires_member_scope(key) {
        return Ok(key.to_string());
    }

    let resolved_scope = resolve_member_scope(conn, scope)?;
    Ok(build_member_scoped_key(
        &resolved_scope.owner_user_id,
        &resolved_scope.member_id,
        key,
    ))
}

fn load_config_value(
    conn: &rusqlite::Connection,
    key: &str,
    scope: Option<MemberScopeInput>,
) -> Result<String, String> {
    let effective_key = resolve_effective_key(conn, key, scope)?;
    let result = conn.query_row(
        "SELECT config_value FROM system_config WHERE config_key = ?1",
        [&effective_key],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(value) => Ok(value),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(String::new()),
        Err(e) => Err(format!("读取配置失败: {}", e)),
    }
}

fn persist_config_value(
    conn: &rusqlite::Connection,
    key: &str,
    value: String,
    scope: Option<MemberScopeInput>,
) -> Result<bool, String> {
    let effective_key = resolve_effective_key(conn, key, scope)?;
    let now = chrono::Local::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO system_config (id, config_key, config_value, updated_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(config_key) DO UPDATE SET
            config_value = excluded.config_value,
            updated_at = excluded.updated_at",
        rusqlite::params![id, effective_key, value, now],
    )
    .map_err(|e| format!("保存配置失败: {}", e))?;

    Ok(true)
}

#[tauri::command]
pub fn get_config(
    key: String,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<String, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    load_config_value(conn, &key, scope)
}

#[tauri::command]
pub fn save_config(
    key: String,
    value: String,
    scope: Option<MemberScopeInput>,
    db: State<Database>,
) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    persist_config_value(conn, &key, value, scope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn create_test_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("connection should open");
        conn.execute_batch(
            "
            CREATE TABLE system_config (
                id TEXT PRIMARY KEY,
                config_key TEXT NOT NULL UNIQUE,
                config_value TEXT DEFAULT '',
                updated_at TEXT NOT NULL
            );
            ",
        )
        .expect("schema should initialize");
        conn
    }

    #[test]
    fn member_scoped_config_isolated_between_members() {
        let conn = create_test_conn();

        persist_config_value(
            &conn,
            "user_custom_prompt_template",
            "成员A提示词".to_string(),
            Some(MemberScopeInput {
                owner_user_id: Some("user-1".to_string()),
                member_id: Some("member-a".to_string()),
                member_name: Some("本人".to_string()),
            }),
        )
        .expect("member A config should save");

        persist_config_value(
            &conn,
            "user_custom_prompt_template",
            "成员B提示词".to_string(),
            Some(MemberScopeInput {
                owner_user_id: Some("user-1".to_string()),
                member_id: Some("member-b".to_string()),
                member_name: Some("母亲".to_string()),
            }),
        )
        .expect("member B config should save");

        let value_a = load_config_value(
            &conn,
            "user_custom_prompt_template",
            Some(MemberScopeInput {
                owner_user_id: Some("user-1".to_string()),
                member_id: Some("member-a".to_string()),
                member_name: Some("本人".to_string()),
            }),
        )
        .expect("member A config should load");
        let value_b = load_config_value(
            &conn,
            "user_custom_prompt_template",
            Some(MemberScopeInput {
                owner_user_id: Some("user-1".to_string()),
                member_id: Some("member-b".to_string()),
                member_name: Some("母亲".to_string()),
            }),
        )
        .expect("member B config should load");

        assert_eq!(value_a, "成员A提示词");
        assert_eq!(value_b, "成员B提示词");
    }

    #[test]
    fn member_scoped_config_does_not_fallback_to_global_value() {
        let conn = create_test_conn();

        conn.execute(
            "INSERT INTO system_config (id, config_key, config_value, updated_at)
             VALUES ('global-user-custom', 'user_custom_prompt_template', '全局默认提示词', '2026-04-08T00:00:00+08:00')",
            [],
        )
        .expect("global config should seed");

        let value = load_config_value(
            &conn,
            "user_custom_prompt_template",
            Some(MemberScopeInput {
                owner_user_id: Some("user-1".to_string()),
                member_id: Some("member-a".to_string()),
                member_name: Some("本人".to_string()),
            }),
        )
        .expect("member scoped load should return empty when missing");

        assert_eq!(value, "");
    }
}
