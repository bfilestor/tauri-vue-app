use serde::{Deserialize, Serialize};
use tauri::State;
use crate::db::Database;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigValue {
    pub config_key: String,
    pub config_value: String,
}

#[tauri::command]
pub fn get_config(key: String, db: State<Database>) -> Result<String, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let result = conn.query_row(
        "SELECT config_value FROM system_config WHERE config_key = ?1",
        [&key],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(value) => Ok(value),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(String::new()),
        Err(e) => Err(format!("读取配置失败: {}", e)),
    }
}

#[tauri::command]
pub fn save_config(key: String, value: String, db: State<Database>) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let now = chrono::Local::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO system_config (id, config_key, config_value, updated_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(config_key) DO UPDATE SET
            config_value = excluded.config_value,
            updated_at = excluded.updated_at",
        rusqlite::params![id, key, value, now],
    )
    .map_err(|e| format!("保存配置失败: {}", e))?;

    Ok(true)
}
