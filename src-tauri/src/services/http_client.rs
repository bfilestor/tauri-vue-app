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
    pub timeout: u64,
}

/// 根据配置构建 HTTP 客户端（支持 SOCKS5 代理）
pub fn build_client(config: &AiClientConfig) -> Result<Client, String> {
    let mut builder = Client::builder().timeout(Duration::from_secs(config.timeout));

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

/// 从数据库读取 AI 配置
pub fn load_ai_config(conn: &rusqlite::Connection) -> Result<AiClientConfig, String> {
    let get_config = |key: &str| -> String {
        conn.query_row(
            "SELECT config_value FROM system_config WHERE config_key = ?1",
            [key],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_default()
    };

    let api_url = get_config("ai_api_url");
    let api_key = get_config("ai_api_key");
    let proxy_enabled = get_config("proxy_enabled") == "true";
    let proxy_url = get_config("proxy_url");
    let proxy_username = get_config("proxy_username");
    let proxy_password = get_config("proxy_password");
    let timeout = get_config("ai_timeout").parse::<u64>().unwrap_or(120);

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
        timeout,
    })
}

/// 获取默认模型名称
pub fn get_default_model(conn: &rusqlite::Connection) -> String {
    let default = conn
        .query_row(
            "SELECT config_value FROM system_config WHERE config_key = 'ai_default_model'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_default();

    if !default.is_empty() {
        return default;
    }

    // 回退到模型列表的第一个
    let models_json = conn
        .query_row(
            "SELECT config_value FROM system_config WHERE config_key = 'ai_models'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_default();

    if let Ok(models) = serde_json::from_str::<Vec<String>>(&models_json) {
        models
            .into_iter()
            .next()
            .unwrap_or_else(|| "gpt-4o-mini".to_string())
    } else {
        "gpt-4o-mini".to_string()
    }
}
