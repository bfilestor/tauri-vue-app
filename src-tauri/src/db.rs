use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    /// 初始化数据库，在 app_dir 下创建 health_guard.db
    pub fn new(app_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_dir).ok();
        let db_path = app_dir.join("health_guard.db");
        let conn = Connection::open(db_path)?;

        // 启用 WAL 模式提升并发性能
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Database {
            conn: Mutex::new(conn),
        };
        db.init_tables()?;
        Ok(db)
    }

    /// 创建全部 8 张表
    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
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
            "
        )?;
        Ok(())
    }
}
