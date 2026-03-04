use reqwest::Client;
use std::time::Duration;

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

/// 从新的 ai_providers/ai_models 表加载默认提供商配置
/// 优先使用 is_default=1 的模型所属的 Provider；fallback 到第一个 enabled=1 的 Provider
pub fn load_default_provider_config(conn: &rusqlite::Connection) -> Result<AiClientConfig, String> {
    // 尝试从 ai_models 的 is_default=1 找到对应 provider
    let result: Result<(String, String), _> = conn.query_row(
        "SELECT p.api_url, p.api_key
         FROM ai_models m
         JOIN ai_providers p ON m.provider_id = p.id
         WHERE m.is_default = 1 AND p.enabled = 1
         LIMIT 1",
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
        return Err("请先配置 AI API 地址".into());
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

/// 从数据库读取 AI 配置（优先使用新表，fallback 到旧 system_config）
pub fn load_ai_config(conn: &rusqlite::Connection) -> Result<AiClientConfig, String> {
    // 先尝试从新的 ai_providers 表加载
    let provider_count: i32 = conn
        .query_row("SELECT COUNT(*) FROM ai_providers WHERE enabled = 1", [], |row| row.get(0))
        .unwrap_or(0);

    if provider_count > 0 {
        return load_default_provider_config(conn);
    }

    // Fallback: 旧版 system_config 方式
    let api_url = get_system_config(conn, "ai_api_url");
    let api_key = get_system_config(conn, "ai_api_key");
    let (proxy_enabled, proxy_url, proxy_username, proxy_password, timeout) = load_network_config(conn);

    if api_url.is_empty() {
        return Err("请先配置 AI API 地址".into());
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

/// 获取默认模型名称（优先使用新表，fallback 到旧 system_config）
pub fn get_default_model(conn: &rusqlite::Connection) -> String {
    // 先从新的 ai_models 表查找 is_default=1 的模型
    let new_default = conn.query_row(
        "SELECT m.model_id FROM ai_models m
         JOIN ai_providers p ON m.provider_id = p.id
         WHERE m.is_default = 1 AND p.enabled = 1
         LIMIT 1",
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

    // Fallback: 旧版 system_config
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
