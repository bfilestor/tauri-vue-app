use reqwest::Client;
use std::time::Duration;

const MODEL_SCENE_OCR: &str = "ocr";
const MODEL_SCENE_ANALYSIS: &str = "analysis";

/// AI API 配置结构
pub struct AiClientConfig {
    pub api_url: String,
    pub api_key: String,
    pub proxy_enabled: bool,
    pub proxy_url: String,
    pub proxy_username: String,
    pub proxy_password: String,
    pub timeout_secs: u64,
}

/// 根据配置构建 HTTP 客户端（支持 SOCKS5 代理）
pub fn build_client(config: &AiClientConfig) -> Result<Client, String> {
    let mut builder = Client::builder().timeout(Duration::from_secs(config.timeout_secs));

    if config.proxy_enabled && !config.proxy_url.is_empty() {
        let proxy_addr =
            if config.proxy_url.starts_with("socks5://") || config.proxy_url.starts_with("http") {
                config.proxy_url.clone()
            } else {
                format!("socks5://{}", config.proxy_url)
            };

        let mut proxy =
            reqwest::Proxy::all(&proxy_addr).map_err(|e| format!("代理地址无效: {}", e))?;

        if !config.proxy_username.is_empty() {
            proxy = proxy.basic_auth(&config.proxy_username, &config.proxy_password);
        }

        builder = builder.proxy(proxy);
    }

    builder
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))
}

/// 辅助函数：从 system_config 表读取配置值
fn get_system_config(conn: &rusqlite::Connection, key: &str) -> String {
    conn.query_row(
        "SELECT config_value FROM system_config WHERE config_key = ?1",
        [key],
        |row| row.get::<_, String>(0),
    )
    .unwrap_or_default()
}

/// 加载全局代理和超时设置（公共部分）
fn load_network_config(conn: &rusqlite::Connection) -> (bool, String, String, String, u64) {
    let proxy_enabled = get_system_config(conn, "proxy_enabled") == "true";
    let proxy_url = get_system_config(conn, "proxy_url");
    let proxy_username = get_system_config(conn, "proxy_username");
    let proxy_password = get_system_config(conn, "proxy_password");
    let timeout = get_system_config(conn, "ai_timeout").parse::<u64>().unwrap_or(120);
    (proxy_enabled, proxy_url, proxy_username, proxy_password, timeout)
}

fn scene_default_column(scene: &str) -> &'static str {
    match scene {
        MODEL_SCENE_OCR => "is_default_ocr",
        MODEL_SCENE_ANALYSIS => "is_default_analysis",
        _ => "is_default_analysis",
    }
}

fn scene_label(scene: &str) -> &'static str {
    match scene {
        MODEL_SCENE_OCR => "OCR",
        MODEL_SCENE_ANALYSIS => "分析",
        _ => "分析",
    }
}

/// 从新的 ai_providers/ai_models 表加载默认提供商配置
/// 优先使用场景默认模型所属 Provider；fallback 到第一个 enabled=1 的 Provider。
pub fn load_default_provider_config_for_scene(
    conn: &rusqlite::Connection,
    scene: &str,
) -> Result<AiClientConfig, String> {
    let default_column = scene_default_column(scene);
    let sql = format!(
        "SELECT p.api_url, p.api_key
         FROM ai_models m
         JOIN ai_providers p ON m.provider_id = p.id
         WHERE m.{default_column} = 1 AND m.enabled = 1 AND p.enabled = 1
         LIMIT 1"
    );

    let result: Result<(String, String), _> = conn.query_row(
        &sql,
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    let (api_url, api_key) = match result {
        Ok(r) => r,
        Err(_) => {
            // Fallback: 第一个 enabled 的 provider
            conn.query_row(
                "SELECT api_url, api_key FROM ai_providers WHERE enabled = 1 ORDER BY sort_order ASC LIMIT 1",
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .map_err(|_| "未找到可用的 AI 提供商，请先添加并配置提供商".to_string())?
        }
    };

    if api_url.is_empty() {
        return Err(format!("请先配置{}场景 AI API 地址", scene_label(scene)));
    }

    let (proxy_enabled, proxy_url, proxy_username, proxy_password, timeout) = load_network_config(conn);

    Ok(AiClientConfig {
        api_url,
        api_key,
        proxy_enabled,
        proxy_url,
        proxy_username,
        proxy_password,
        timeout_secs: timeout,
    })
}

fn load_ai_config_for_scene(conn: &rusqlite::Connection, scene: &str) -> Result<AiClientConfig, String> {
    // 先尝试从新的 ai_providers 表加载
    let provider_count: i32 = conn
        .query_row("SELECT COUNT(*) FROM ai_providers WHERE enabled = 1", [], |row| row.get(0))
        .unwrap_or(0);

    if provider_count > 0 {
        return load_default_provider_config_for_scene(conn, scene);
    }

    // Fallback: 旧版 system_config 方式
    let api_url = get_system_config(conn, "ai_api_url");
    let api_key = get_system_config(conn, "ai_api_key");
    let (proxy_enabled, proxy_url, proxy_username, proxy_password, timeout) = load_network_config(conn);

    if api_url.is_empty() {
        return Err(format!("请先配置{}场景 AI API 地址", scene_label(scene)));
    }
    if api_key.is_empty() {
        return Err("请先配置 API Key".into());
    }

    Ok(AiClientConfig {
        api_url,
        api_key,
        proxy_enabled,
        proxy_url,
        proxy_username,
        proxy_password,
        timeout_secs: timeout,
    })
}

pub fn load_ai_config_for_ocr(conn: &rusqlite::Connection) -> Result<AiClientConfig, String> {
    load_ai_config_for_scene(conn, MODEL_SCENE_OCR)
}

pub fn load_ai_config_for_analysis(conn: &rusqlite::Connection) -> Result<AiClientConfig, String> {
    load_ai_config_for_scene(conn, MODEL_SCENE_ANALYSIS)
}

/// 从数据库读取 AI 配置（优先使用新表，fallback 到旧 system_config）
pub fn load_ai_config(conn: &rusqlite::Connection) -> Result<AiClientConfig, String> {
    load_ai_config_for_analysis(conn)
}

fn get_default_model_for_scene(conn: &rusqlite::Connection, scene: &str) -> String {
    let default_column = scene_default_column(scene);
    let sql = format!(
        "SELECT m.model_id
         FROM ai_models m
         JOIN ai_providers p ON m.provider_id = p.id
         WHERE m.{default_column} = 1 AND m.enabled = 1 AND p.enabled = 1
         LIMIT 1"
    );

    let new_default = conn.query_row(
        &sql,
        [],
        |row| row.get::<_, String>(0),
    );

    if let Ok(model) = new_default {
        if !model.is_empty() {
            return model;
        }
    }

    // Fallback: 从新表取第一个 enabled 的模型
    let first_model = conn.query_row(
        "SELECT m.model_id FROM ai_models m
         JOIN ai_providers p ON m.provider_id = p.id
         WHERE m.enabled = 1 AND p.enabled = 1
         ORDER BY m.sort_order ASC
         LIMIT 1",
        [],
        |row| row.get::<_, String>(0),
    );

    if let Ok(model) = first_model {
        if !model.is_empty() {
            return model;
        }
    }

    // Fallback: 场景配置/旧版 system_config
    let scene_config_key = match scene {
        MODEL_SCENE_OCR => "ai_ocr_default_model",
        MODEL_SCENE_ANALYSIS => "ai_analysis_default_model",
        _ => "ai_analysis_default_model",
    };
    let scene_default = get_system_config(conn, scene_config_key);
    if !scene_default.is_empty() {
        return scene_default;
    }

    let default = get_system_config(conn, "ai_default_model");
    if !default.is_empty() {
        return default;
    }

    // 回退到模型列表的第一个
    let models_json = get_system_config(conn, "ai_models");

    if let Ok(models) = serde_json::from_str::<Vec<String>>(&models_json) {
        models
            .into_iter()
            .next()
            .unwrap_or_else(|| "gpt-4o-mini".to_string())
    } else {
        "gpt-4o-mini".to_string()
    }
}

pub fn get_default_model_for_ocr(conn: &rusqlite::Connection) -> String {
    get_default_model_for_scene(conn, MODEL_SCENE_OCR)
}

pub fn get_default_model_for_analysis(conn: &rusqlite::Connection) -> String {
    get_default_model_for_scene(conn, MODEL_SCENE_ANALYSIS)
}

/// 获取默认模型名称（兼容旧调用，默认 analysis 场景）
pub fn get_default_model(conn: &rusqlite::Connection) -> String {
    get_default_model_for_analysis(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_database(name: &str) -> (Database, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "health-monitor-http-client-tests-{}-{}",
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

    #[test]
    fn scene_specific_config_and_model_resolution_are_isolated() {
        let (db, dir) = create_test_database("scene-routing");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");

            conn.execute(
                "INSERT INTO ai_providers (id, name, type, api_key, api_url, enabled, sort_order, created_at, updated_at)
                 VALUES ('p-analysis', 'analysis-provider', 'openai', 'k-analysis', 'https://analysis.example/v1/chat/completions', 1, 0, '2026-04-13T00:00:00+08:00', '2026-04-13T00:00:00+08:00')",
                [],
            )
            .expect("insert analysis provider should succeed");
            conn.execute(
                "INSERT INTO ai_providers (id, name, type, api_key, api_url, enabled, sort_order, created_at, updated_at)
                 VALUES ('p-ocr', 'ocr-provider', 'openai', 'k-ocr', 'https://ocr.example/v1/chat/completions', 1, 1, '2026-04-13T00:00:00+08:00', '2026-04-13T00:00:00+08:00')",
                [],
            )
            .expect("insert ocr provider should succeed");

            conn.execute(
                "INSERT INTO ai_models (id, provider_id, model_id, model_name, group_name, is_default, is_default_ocr, is_default_analysis, enabled, sort_order, created_at)
                 VALUES ('m-analysis', 'p-analysis', 'analysis-model', 'analysis-model', '', 1, 0, 1, 1, 0, '2026-04-13T00:00:00+08:00')",
                [],
            )
            .expect("insert analysis model should succeed");
            conn.execute(
                "INSERT INTO ai_models (id, provider_id, model_id, model_name, group_name, is_default, is_default_ocr, is_default_analysis, enabled, sort_order, created_at)
                 VALUES ('m-ocr', 'p-ocr', 'ocr-model', 'ocr-model', '', 0, 1, 0, 1, 0, '2026-04-13T00:00:00+08:00')",
                [],
            )
            .expect("insert ocr model should succeed");

            let ocr_config = load_ai_config_for_ocr(conn).expect("load ocr config should succeed");
            let analysis_config =
                load_ai_config_for_analysis(conn).expect("load analysis config should succeed");
            let ocr_model = get_default_model_for_ocr(conn);
            let analysis_model = get_default_model_for_analysis(conn);

            assert_eq!(ocr_config.api_url, "https://ocr.example/v1/chat/completions");
            assert_eq!(
                analysis_config.api_url,
                "https://analysis.example/v1/chat/completions"
            );
            assert_eq!(ocr_model, "ocr-model");
            assert_eq!(analysis_model, "analysis-model");
        }

        cleanup_test_database(&db, dir);
    }
}
