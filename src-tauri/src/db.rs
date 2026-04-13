use rusqlite::{Connection, OptionalExtension, Result, Transaction};
use std::path::PathBuf;
use std::sync::Mutex;

const SCHEMA_VERSION_V1: i32 = 1;
const SCHEMA_VERSION_V2: i32 = 2;
const SCHEMA_VERSION_V3: i32 = 3;
const SCHEMA_VERSION_V4: i32 = 4;

#[allow(dead_code)]
pub const CONFIG_KEY_ACTIVE_OWNER_USER_ID: &str = "health.activeOwnerUserId";
#[allow(dead_code)]
pub const CONFIG_KEY_ACTIVE_MEMBER_ID: &str = "health.activeMemberId";
#[allow(dead_code)]
pub const CONFIG_KEY_ACTIVE_CONVERSATION_ID: &str = "health.activeConversationId";

pub struct Database {
    pub conn: Mutex<Option<Connection>>,
    pub db_path: PathBuf,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberScope {
    pub owner_user_id: String,
    pub member_id: String,
}

#[allow(dead_code)]
fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let trimmed = item.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

#[allow(dead_code)]
pub fn read_optional_config(conn: &Connection, key: &str) -> Result<Option<String>> {
    let value = conn
        .query_row(
            "SELECT config_value FROM system_config WHERE config_key = ?1",
            [key],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    Ok(normalize_optional_text(value))
}

#[allow(dead_code)]
pub fn read_active_member_scope(conn: &Connection) -> Result<Option<MemberScope>> {
    let owner_user_id = read_optional_config(conn, CONFIG_KEY_ACTIVE_OWNER_USER_ID)?;
    let member_id = read_optional_config(conn, CONFIG_KEY_ACTIVE_MEMBER_ID)?;

    match (owner_user_id, member_id) {
        (Some(owner_user_id), Some(member_id)) => Ok(Some(MemberScope {
            owner_user_id,
            member_id,
        })),
        _ => Ok(None),
    }
}

#[allow(dead_code)]
pub fn read_active_conversation_id(conn: &Connection) -> Result<Option<String>> {
    read_optional_config(conn, CONFIG_KEY_ACTIVE_CONVERSATION_ID)
}

impl Database {
    /// 初始化数据库，在 app_dir 下创建 health_guard.db
    pub fn new(app_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_dir).ok();
        let db_path = app_dir.join("health_guard.db");
        let mut conn = Connection::open(&db_path)?;

        // 启用 WAL 模式提升并发性能
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        initialize_schema(&mut conn)?;

        let db = Database {
            conn: Mutex::new(Some(conn)),
            db_path,
        };
        Ok(db)
    }

    pub fn close(&self) -> Result<(), String> {
        let mut conn_guard = self.conn.lock().map_err(|e| e.to_string())?;
        *conn_guard = None;
        Ok(())
    }

    pub fn reopen(&self) -> Result<(), String> {
        let mut conn_guard = self.conn.lock().map_err(|e| e.to_string())?;
        if conn_guard.is_none() {
            let mut conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;
            conn.execute_batch("PRAGMA journal_mode=WAL;")
                .map_err(|e| e.to_string())?;
            conn.execute_batch("PRAGMA foreign_keys=ON;")
                .map_err(|e| e.to_string())?;
            initialize_schema(&mut conn).map_err(|e| e.to_string())?;
            *conn_guard = Some(conn);
        }
        Ok(())
    }
}

fn initialize_schema(conn: &mut Connection) -> Result<()> {
    let current_version = get_user_version(conn)?;

    if current_version >= SCHEMA_VERSION_V4 {
        let tx = conn.transaction()?;
        apply_v2_multi_user_schema(&tx)?;
        apply_v3_member_scoped_project_schema(&tx)?;
        apply_v4_ai_model_scene_defaults_schema(&tx)?;
        set_user_version(&tx, SCHEMA_VERSION_V4)?;
        tx.commit()?;
        return Ok(());
    }

    let tx = conn.transaction()?;

    if current_version < SCHEMA_VERSION_V1 {
        apply_v1_baseline(&tx)?;
        set_user_version(&tx, SCHEMA_VERSION_V1)?;
    }

    if current_version < SCHEMA_VERSION_V2 {
        apply_v2_multi_user_schema(&tx)?;
        set_user_version(&tx, SCHEMA_VERSION_V2)?;
    }

    if current_version < SCHEMA_VERSION_V3 {
        apply_v3_member_scoped_project_schema(&tx)?;
        set_user_version(&tx, SCHEMA_VERSION_V3)?;
    }

    if current_version < SCHEMA_VERSION_V4 {
        apply_v4_ai_model_scene_defaults_schema(&tx)?;
        set_user_version(&tx, SCHEMA_VERSION_V4)?;
    }

    tx.commit()?;
    Ok(())
}

fn get_user_version(conn: &Connection) -> Result<i32> {
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
}

fn set_user_version(tx: &Transaction<'_>, version: i32) -> Result<()> {
    tx.execute_batch(&format!("PRAGMA user_version = {version};"))
}

fn apply_v1_baseline(tx: &Transaction<'_>) -> Result<()> {
    tx.execute_batch(
        "
        -- 1. 检查项目表
        CREATE TABLE IF NOT EXISTS checkup_projects (
            id              TEXT PRIMARY KEY,
            name            TEXT NOT NULL,
            description     TEXT DEFAULT '',
            sort_order      INTEGER DEFAULT 0,
            is_active       INTEGER DEFAULT 1,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL
        );

        -- 2. 检查指标表
        CREATE TABLE IF NOT EXISTS indicators (
            id              TEXT PRIMARY KEY,
            project_id      TEXT NOT NULL,
            name            TEXT NOT NULL,
            unit            TEXT DEFAULT '',
            reference_range TEXT DEFAULT '',
            sort_order      INTEGER DEFAULT 0,
            is_core         INTEGER DEFAULT 0,
            created_at      TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES checkup_projects(id)
        );

        -- 3. 检查记录表
        CREATE TABLE IF NOT EXISTS checkup_records (
            id              TEXT PRIMARY KEY,
            checkup_date    TEXT NOT NULL,
            status          TEXT NOT NULL DEFAULT 'pending_ocr',
            notes           TEXT DEFAULT '',
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL
        );

        -- 4. 检查文件表
        CREATE TABLE IF NOT EXISTS checkup_files (
            id                TEXT PRIMARY KEY,
            record_id         TEXT NOT NULL,
            project_id        TEXT NOT NULL,
            original_filename TEXT NOT NULL,
            stored_path       TEXT NOT NULL,
            file_size         INTEGER DEFAULT 0,
            mime_type         TEXT DEFAULT '',
            uploaded_at       TEXT NOT NULL,
            FOREIGN KEY (record_id) REFERENCES checkup_records(id),
            FOREIGN KEY (project_id) REFERENCES checkup_projects(id)
        );

        -- 5. OCR 结果表
        CREATE TABLE IF NOT EXISTS ocr_results (
            id              TEXT PRIMARY KEY,
            file_id         TEXT NOT NULL,
            record_id       TEXT NOT NULL,
            project_id      TEXT NOT NULL,
            checkup_date    TEXT NOT NULL,
            raw_json        TEXT DEFAULT '',
            parsed_items    TEXT DEFAULT '[]',
            status          TEXT NOT NULL DEFAULT 'processing',
            error_message   TEXT DEFAULT '',
            created_at      TEXT NOT NULL,
            FOREIGN KEY (file_id) REFERENCES checkup_files(id),
            FOREIGN KEY (record_id) REFERENCES checkup_records(id),
            FOREIGN KEY (project_id) REFERENCES checkup_projects(id)
        );

        -- 6. AI 分析记录表
        CREATE TABLE IF NOT EXISTS ai_analyses (
            id                TEXT PRIMARY KEY,
            record_id         TEXT NOT NULL,
            request_prompt    TEXT DEFAULT '',
            response_content  TEXT DEFAULT '',
            model_used        TEXT DEFAULT '',
            status            TEXT NOT NULL DEFAULT 'processing',
            error_message     TEXT DEFAULT '',
            created_at        TEXT NOT NULL,
            FOREIGN KEY (record_id) REFERENCES checkup_records(id)
        );

        -- 7. 指标值表（用于趋势分析）
        CREATE TABLE IF NOT EXISTS indicator_values (
            id              TEXT PRIMARY KEY,
            ocr_result_id   TEXT NOT NULL,
            record_id       TEXT NOT NULL,
            project_id      TEXT NOT NULL,
            indicator_id    TEXT NOT NULL,
            checkup_date    TEXT NOT NULL,
            value           REAL,
            value_text      TEXT DEFAULT '',
            is_abnormal     INTEGER DEFAULT 0,
            created_at      TEXT NOT NULL,
            FOREIGN KEY (ocr_result_id) REFERENCES ocr_results(id),
            FOREIGN KEY (record_id) REFERENCES checkup_records(id),
            FOREIGN KEY (project_id) REFERENCES checkup_projects(id),
            FOREIGN KEY (indicator_id) REFERENCES indicators(id)
        );

        -- 8. 系统配置表
        CREATE TABLE IF NOT EXISTS system_config (
            id              TEXT PRIMARY KEY,
            config_key      TEXT NOT NULL UNIQUE,
            config_value    TEXT DEFAULT '',
            updated_at      TEXT NOT NULL
        );

        -- 9. 聊天记录表
        CREATE TABLE IF NOT EXISTS chat_logs (
            id              TEXT PRIMARY KEY,
            role            TEXT NOT NULL,
            content         TEXT NOT NULL,
            created_at      TEXT NOT NULL
        );

        -- 10. AI 提供商表
        CREATE TABLE IF NOT EXISTS ai_providers (
            id              TEXT PRIMARY KEY,
            name            TEXT NOT NULL,
            type            TEXT NOT NULL DEFAULT 'openai',
            api_key         TEXT DEFAULT '',
            api_url         TEXT DEFAULT '',
            enabled         INTEGER DEFAULT 1,
            sort_order      INTEGER DEFAULT 0,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL
        );

        -- 11. AI 模型表
        CREATE TABLE IF NOT EXISTS ai_models (
            id              TEXT PRIMARY KEY,
            provider_id     TEXT NOT NULL,
            model_id        TEXT NOT NULL,
            model_name      TEXT DEFAULT '',
            group_name      TEXT DEFAULT '',
            is_default      INTEGER DEFAULT 0,
            enabled         INTEGER DEFAULT 1,
            sort_order      INTEGER DEFAULT 0,
            created_at      TEXT NOT NULL,
            FOREIGN KEY (provider_id) REFERENCES ai_providers(id)
        );
        ",
    )?;

    tx.execute_batch(
        "
        CREATE INDEX IF NOT EXISTS idx_indicators_project ON indicators(project_id, sort_order, name);
        CREATE INDEX IF NOT EXISTS idx_checkup_files_record ON checkup_files(record_id, project_id, uploaded_at);
        CREATE INDEX IF NOT EXISTS idx_ocr_results_record ON ocr_results(record_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_ai_analyses_record ON ai_analyses(record_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_indicator_values_indicator_date ON indicator_values(indicator_id, checkup_date);
        CREATE INDEX IF NOT EXISTS idx_chat_logs_created_at ON chat_logs(created_at);
        ",
    )?;

    Ok(())
}

fn apply_v2_multi_user_schema(tx: &Transaction<'_>) -> Result<()> {
    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS local_users (
            id                  TEXT PRIMARY KEY,
            cloud_user_id       TEXT NOT NULL UNIQUE,
            user_name           TEXT DEFAULT '',
            nick_name           TEXT DEFAULT '',
            phone               TEXT DEFAULT '',
            avatar_url          TEXT DEFAULT '',
            status              TEXT NOT NULL DEFAULT 'ACTIVE',
            last_login_at       TEXT DEFAULT '',
            created_at          TEXT NOT NULL,
            updated_at          TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS family_members (
            id                  TEXT PRIMARY KEY,
            cloud_member_id     TEXT NOT NULL,
            owner_user_id       TEXT NOT NULL,
            member_name         TEXT NOT NULL,
            relation_code       TEXT NOT NULL,
            gender              TEXT DEFAULT '',
            birthday            TEXT DEFAULT '',
            mobile              TEXT DEFAULT '',
            health_note         TEXT DEFAULT '',
            is_default          INTEGER DEFAULT 0,
            status              TEXT NOT NULL DEFAULT 'ENABLED',
            created_at          TEXT NOT NULL,
            updated_at          TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS chat_conversations (
            id                  TEXT PRIMARY KEY,
            owner_user_id       TEXT NOT NULL,
            member_id           TEXT NOT NULL,
            title               TEXT DEFAULT '',
            created_at          TEXT NOT NULL,
            updated_at          TEXT NOT NULL
        );
        ",
    )?;

    add_column_if_missing(
        tx,
        "checkup_records",
        "owner_user_id TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(tx, "checkup_records", "member_id TEXT NOT NULL DEFAULT ''")?;
    add_column_if_missing(
        tx,
        "checkup_records",
        "member_name_snapshot TEXT DEFAULT ''",
    )?;

    add_column_if_missing(
        tx,
        "checkup_files",
        "owner_user_id TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(tx, "checkup_files", "member_id TEXT NOT NULL DEFAULT ''")?;

    add_column_if_missing(tx, "ocr_results", "owner_user_id TEXT NOT NULL DEFAULT ''")?;
    add_column_if_missing(tx, "ocr_results", "member_id TEXT NOT NULL DEFAULT ''")?;

    add_column_if_missing(tx, "ai_analyses", "owner_user_id TEXT NOT NULL DEFAULT ''")?;
    add_column_if_missing(tx, "ai_analyses", "member_id TEXT NOT NULL DEFAULT ''")?;

    add_column_if_missing(
        tx,
        "indicator_values",
        "owner_user_id TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(tx, "indicator_values", "member_id TEXT NOT NULL DEFAULT ''")?;

    add_column_if_missing(tx, "chat_logs", "owner_user_id TEXT NOT NULL DEFAULT ''")?;
    add_column_if_missing(tx, "chat_logs", "member_id TEXT NOT NULL DEFAULT ''")?;
    add_column_if_missing(tx, "chat_logs", "conversation_id TEXT NOT NULL DEFAULT ''")?;

    tx.execute_batch(
        "
        CREATE UNIQUE INDEX IF NOT EXISTS idx_local_users_cloud_user_id
            ON local_users(cloud_user_id);
        CREATE INDEX IF NOT EXISTS idx_local_users_last_login_at
            ON local_users(last_login_at);

        CREATE UNIQUE INDEX IF NOT EXISTS idx_family_members_owner_cloud_member
            ON family_members(owner_user_id, cloud_member_id);
        CREATE INDEX IF NOT EXISTS idx_family_members_owner_default
            ON family_members(owner_user_id, is_default, status);

        CREATE INDEX IF NOT EXISTS idx_chat_conversations_owner_member_updated
            ON chat_conversations(owner_user_id, member_id, updated_at DESC);

        CREATE INDEX IF NOT EXISTS idx_checkup_records_owner_member_date
            ON checkup_records(owner_user_id, member_id, checkup_date DESC, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_checkup_files_owner_member_record
            ON checkup_files(owner_user_id, member_id, record_id, uploaded_at);
        CREATE INDEX IF NOT EXISTS idx_ocr_results_owner_member_created
            ON ocr_results(owner_user_id, member_id, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_ai_analyses_owner_member_created
            ON ai_analyses(owner_user_id, member_id, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_indicator_values_owner_member_date
            ON indicator_values(owner_user_id, member_id, project_id, indicator_id, checkup_date);
        CREATE INDEX IF NOT EXISTS idx_chat_logs_owner_member_conversation_created
            ON chat_logs(owner_user_id, member_id, conversation_id, created_at DESC);
        ",
    )?;

    Ok(())
}

fn apply_v3_member_scoped_project_schema(tx: &Transaction<'_>) -> Result<()> {
    add_column_if_missing(
        tx,
        "checkup_projects",
        "owner_user_id TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(tx, "checkup_projects", "member_id TEXT NOT NULL DEFAULT ''")?;

    add_column_if_missing(tx, "indicators", "owner_user_id TEXT NOT NULL DEFAULT ''")?;
    add_column_if_missing(tx, "indicators", "member_id TEXT NOT NULL DEFAULT ''")?;

    tx.execute_batch(
        "
        CREATE INDEX IF NOT EXISTS idx_checkup_projects_owner_member_sort
            ON checkup_projects(owner_user_id, member_id, is_active, sort_order, created_at);
        CREATE INDEX IF NOT EXISTS idx_indicators_owner_member_project_sort
            ON indicators(owner_user_id, member_id, project_id, sort_order, created_at);
        ",
    )?;

    Ok(())
}

fn apply_v4_ai_model_scene_defaults_schema(tx: &Transaction<'_>) -> Result<()> {
    add_column_if_missing(
        tx,
        "ai_models",
        "is_default_ocr INTEGER NOT NULL DEFAULT 0",
    )?;
    add_column_if_missing(
        tx,
        "ai_models",
        "is_default_analysis INTEGER NOT NULL DEFAULT 0",
    )?;

    tx.execute_batch(
        "
        CREATE INDEX IF NOT EXISTS idx_ai_models_default_ocr
            ON ai_models(is_default_ocr, enabled, sort_order, created_at);
        CREATE INDEX IF NOT EXISTS idx_ai_models_default_analysis
            ON ai_models(is_default_analysis, enabled, sort_order, created_at);
        ",
    )?;

    let analysis_default_count: i32 = tx.query_row(
        "SELECT COUNT(*) FROM ai_models WHERE is_default_analysis = 1",
        [],
        |row| row.get(0),
    )?;

    if analysis_default_count == 0 {
        tx.execute(
            "UPDATE ai_models
             SET is_default_analysis = 1
             WHERE is_default = 1",
            [],
        )?;

        let analysis_default_count_after_legacy_copy: i32 = tx.query_row(
            "SELECT COUNT(*) FROM ai_models WHERE is_default_analysis = 1",
            [],
            |row| row.get(0),
        )?;

        if analysis_default_count_after_legacy_copy == 0 {
            tx.execute(
                "UPDATE ai_models
                 SET is_default_analysis = 1
                 WHERE id = (
                    SELECT id
                    FROM ai_models
                    WHERE enabled = 1
                    ORDER BY sort_order ASC, created_at ASC
                    LIMIT 1
                 )",
                [],
            )?;
        }
    }

    // 保持旧字段向前兼容：is_default 始终与 analysis 场景默认一致。
    tx.execute(
        "UPDATE ai_models
         SET is_default = CASE WHEN is_default_analysis = 1 THEN 1 ELSE 0 END",
        [],
    )?;

    let ocr_default_count: i32 = tx.query_row(
        "SELECT COUNT(*) FROM ai_models WHERE is_default_ocr = 1",
        [],
        |row| row.get(0),
    )?;

    if ocr_default_count == 0 {
        tx.execute(
            "UPDATE ai_models
             SET is_default_ocr = 1
             WHERE is_default_analysis = 1",
            [],
        )?;

        let ocr_default_count_after_copy: i32 = tx.query_row(
            "SELECT COUNT(*) FROM ai_models WHERE is_default_ocr = 1",
            [],
            |row| row.get(0),
        )?;

        if ocr_default_count_after_copy == 0 {
            tx.execute(
                "UPDATE ai_models
                 SET is_default_ocr = 1
                 WHERE id = (
                    SELECT id
                    FROM ai_models
                    WHERE enabled = 1
                    ORDER BY sort_order ASC, created_at ASC
                    LIMIT 1
                 )",
                [],
            )?;
        }
    }

    Ok(())
}

fn add_column_if_missing(
    tx: &Transaction<'_>,
    table_name: &str,
    column_definition: &str,
) -> Result<()> {
    let column_name = column_definition
        .split_whitespace()
        .next()
        .ok_or_else(|| rusqlite::Error::InvalidColumnName(column_definition.to_string()))?;

    if has_column(tx, table_name, column_name)? {
        return Ok(());
    }

    tx.execute_batch(&format!(
        "ALTER TABLE {table_name} ADD COLUMN {column_definition};"
    ))?;
    Ok(())
}

fn has_column(tx: &Transaction<'_>, table_name: &str, column_name: &str) -> Result<bool> {
    let pragma_sql = format!("PRAGMA table_info({table_name})");
    let mut stmt = tx.prepare(&pragma_sql)?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;

    for row in rows {
        if row? == column_name {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_exists(conn: &Connection, table_name: &str) -> Result<bool> {
        conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
            [table_name],
            |row| row.get::<_, i32>(0),
        )
        .map(|exists| exists == 1)
    }

    fn column_exists(conn: &Connection, table_name: &str, column_name: &str) -> Result<bool> {
        let pragma_sql = format!("PRAGMA table_info({table_name})");
        let mut stmt = conn.prepare(&pragma_sql)?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;

        for row in rows {
            if row? == column_name {
                return Ok(true);
            }
        }

        Ok(false)
    }

    #[test]
    fn initialize_schema_bootstraps_v3_tables_for_new_database() -> Result<()> {
        let mut conn = Connection::open_in_memory()?;

        initialize_schema(&mut conn)?;

        assert_eq!(get_user_version(&conn)?, 4);
        assert!(table_exists(&conn, "local_users")?);
        assert!(table_exists(&conn, "family_members")?);
        assert!(table_exists(&conn, "chat_conversations")?);
        assert!(column_exists(&conn, "ai_models", "is_default_ocr")?);
        assert!(column_exists(&conn, "ai_models", "is_default_analysis")?);
        assert!(column_exists(&conn, "checkup_projects", "owner_user_id")?);
        assert!(column_exists(&conn, "checkup_projects", "member_id")?);
        assert!(column_exists(&conn, "indicators", "owner_user_id")?);
        assert!(column_exists(&conn, "indicators", "member_id")?);
        assert!(column_exists(&conn, "checkup_records", "owner_user_id")?);
        assert!(column_exists(&conn, "checkup_records", "member_id")?);
        assert!(column_exists(
            &conn,
            "checkup_records",
            "member_name_snapshot"
        )?);
        assert!(column_exists(&conn, "chat_logs", "conversation_id")?);
        assert!(column_exists(&conn, "chat_logs", "owner_user_id")?);
        assert!(column_exists(&conn, "chat_logs", "member_id")?);

        Ok(())
    }

    #[test]
    fn initialize_schema_migrates_legacy_database_without_touching_old_rows() -> Result<()> {
        let mut conn = Connection::open_in_memory()?;
        {
            let tx = conn.transaction()?;
            apply_v1_baseline(&tx)?;
            tx.commit()?;
        }

        conn.execute(
            "INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at)
             VALUES ('legacy-record', '2026-04-08', 'pending_upload', '', '2026-04-08T10:00:00+08:00', '2026-04-08T10:00:00+08:00')",
            [],
        )?;
        conn.execute(
            "INSERT INTO chat_logs (id, role, content, created_at)
             VALUES ('legacy-chat', 'user', 'hello', '2026-04-08T10:00:00+08:00')",
            [],
        )?;

        initialize_schema(&mut conn)?;

        assert_eq!(get_user_version(&conn)?, 4);
        assert!(table_exists(&conn, "local_users")?);
        assert!(column_exists(&conn, "ai_models", "is_default_ocr")?);
        assert!(column_exists(&conn, "ai_models", "is_default_analysis")?);
        assert!(column_exists(&conn, "checkup_projects", "owner_user_id")?);
        assert!(column_exists(&conn, "indicators", "member_id")?);
        assert!(column_exists(&conn, "ocr_results", "owner_user_id")?);
        assert!(column_exists(&conn, "indicator_values", "member_id")?);
        assert!(column_exists(&conn, "chat_logs", "conversation_id")?);

        let migrated_owner_user_id: String = conn.query_row(
            "SELECT owner_user_id FROM checkup_records WHERE id = 'legacy-record'",
            [],
            |row| row.get(0),
        )?;
        let migrated_conversation_id: String = conn.query_row(
            "SELECT conversation_id FROM chat_logs WHERE id = 'legacy-chat'",
            [],
            |row| row.get(0),
        )?;

        assert_eq!(migrated_owner_user_id, "");
        assert_eq!(migrated_conversation_id, "");

        Ok(())
    }

    #[test]
    fn read_active_member_scope_requires_complete_context_keys() -> Result<()> {
        let mut conn = Connection::open_in_memory()?;
        initialize_schema(&mut conn)?;

        assert_eq!(read_active_member_scope(&conn)?, None);

        conn.execute(
            "INSERT INTO system_config (id, config_key, config_value, updated_at)
             VALUES ('cfg-1', ?1, '1001', '2026-04-08T10:00:00+08:00')",
            [CONFIG_KEY_ACTIVE_OWNER_USER_ID],
        )?;
        assert_eq!(read_active_member_scope(&conn)?, None);

        conn.execute(
            "INSERT INTO system_config (id, config_key, config_value, updated_at)
             VALUES ('cfg-2', ?1, '2001', '2026-04-08T10:00:00+08:00')",
            [CONFIG_KEY_ACTIVE_MEMBER_ID],
        )?;

        assert_eq!(
            read_active_member_scope(&conn)?,
            Some(MemberScope {
                owner_user_id: "1001".to_string(),
                member_id: "2001".to_string(),
            })
        );

        Ok(())
    }
}
