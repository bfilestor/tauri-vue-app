use crate::db::Database;
use serde::{Deserialize, Serialize};
use tauri::State;

// ===== 数据结构 =====

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiProvider {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub api_key: String,
    pub api_url: String,
    pub enabled: bool,
    pub sort_order: i32,
    pub model_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiModel {
    pub id: String,
    pub provider_id: String,
    pub model_id: String,
    pub model_name: String,
    pub group_name: String,
    pub is_default: bool,
    pub enabled: bool,
    pub sort_order: i32,
    pub created_at: String,
}

// ===== Provider CRUD (ISS-055) =====

#[tauri::command]
pub fn list_providers(db: State<Database>) -> Result<Vec<AiProvider>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.name, p.type, p.api_key, p.api_url, p.enabled, p.sort_order,
                    p.created_at, p.updated_at,
                    (SELECT COUNT(*) FROM ai_models m WHERE m.provider_id = p.id) as model_count
             FROM ai_providers p
             ORDER BY p.sort_order ASC, p.created_at ASC",
        )
        .map_err(|e| format!("查询提供商失败: {}", e))?;

    let providers = stmt
        .query_map([], |row| {
            Ok(AiProvider {
                id: row.get(0)?,
                name: row.get(1)?,
                provider_type: row.get(2)?,
                api_key: row.get(3)?,
                api_url: row.get(4)?,
                enabled: row.get::<_, i32>(5)? == 1,
                sort_order: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                model_count: row.get(9)?,
            })
        })
        .map_err(|e| format!("查询提供商失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析提供商数据失败: {}", e))?;

    Ok(providers)
}

#[tauri::command]
pub fn create_provider(
    name: String,
    provider_type: String,
    db: State<Database>,
) -> Result<AiProvider, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().to_rfc3339();

    // 获取当前最大 sort_order
    let max_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM ai_providers",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO ai_providers (id, name, type, api_key, api_url, enabled, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, '', '', 1, ?4, ?5, ?5)",
        rusqlite::params![id, name, provider_type, max_order + 1, now],
    )
    .map_err(|e| format!("创建提供商失败: {}", e))?;

    Ok(AiProvider {
        id,
        name,
        provider_type,
        api_key: String::new(),
        api_url: String::new(),
        enabled: true,
        sort_order: max_order + 1,
        model_count: 0,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_provider(
    id: String,
    name: Option<String>,
    provider_type: Option<String>,
    api_key: Option<String>,
    api_url: Option<String>,
    enabled: Option<bool>,
    db: State<Database>,
) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    let now = chrono::Local::now().to_rfc3339();

    // 构建动态 UPDATE
    let mut sets = vec!["updated_at = ?1".to_string()];
    let mut param_idx = 2;
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];

    if let Some(ref v) = name {
        sets.push(format!("name = ?{}", param_idx));
        params.push(Box::new(v.clone()));
        param_idx += 1;
    }
    if let Some(ref v) = provider_type {
        sets.push(format!("type = ?{}", param_idx));
        params.push(Box::new(v.clone()));
        param_idx += 1;
    }
    if let Some(ref v) = api_key {
        sets.push(format!("api_key = ?{}", param_idx));
        params.push(Box::new(v.clone()));
        param_idx += 1;
    }
    if let Some(ref v) = api_url {
        sets.push(format!("api_url = ?{}", param_idx));
        params.push(Box::new(v.clone()));
        param_idx += 1;
    }
    if let Some(v) = enabled {
        sets.push(format!("enabled = ?{}", param_idx));
        params.push(Box::new(if v { 1i32 } else { 0i32 }));
        param_idx += 1;
    }

    let sql = format!(
        "UPDATE ai_providers SET {} WHERE id = ?{}",
        sets.join(", "),
        param_idx
    );
    params.push(Box::new(id));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    conn.execute(&sql, param_refs.as_slice())
        .map_err(|e| format!("更新提供商失败: {}", e))?;

    Ok(true)
}

#[tauri::command]
pub fn delete_provider(id: String, db: State<Database>) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    // 级联删除：先删除该提供商下的所有模型
    conn.execute(
        "DELETE FROM ai_models WHERE provider_id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| format!("删除关联模型失败: {}", e))?;

    conn.execute(
        "DELETE FROM ai_providers WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| format!("删除提供商失败: {}", e))?;

    Ok(true)
}

// ===== Model CRUD (ISS-056) =====

#[tauri::command]
pub fn list_provider_models(
    provider_id: String,
    db: State<Database>,
) -> Result<Vec<AiModel>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, provider_id, model_id, model_name, group_name, is_default, enabled, sort_order, created_at
             FROM ai_models
             WHERE provider_id = ?1
             ORDER BY group_name ASC, sort_order ASC, created_at ASC",
        )
        .map_err(|e| format!("查询模型失败: {}", e))?;

    let models = stmt
        .query_map([&provider_id], |row| {
            Ok(AiModel {
                id: row.get(0)?,
                provider_id: row.get(1)?,
                model_id: row.get(2)?,
                model_name: row.get(3)?,
                group_name: row.get(4)?,
                is_default: row.get::<_, i32>(5)? == 1,
                enabled: row.get::<_, i32>(6)? == 1,
                sort_order: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| format!("查询模型失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析模型数据失败: {}", e))?;

    Ok(models)
}

#[tauri::command]
pub fn add_model(
    provider_id: String,
    model_id: String,
    model_name: Option<String>,
    group_name: Option<String>,
    db: State<Database>,
) -> Result<AiModel, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().to_rfc3339();
    let name = model_name.unwrap_or_else(|| model_id.clone());
    let group = group_name.unwrap_or_default();

    // 获取当前最大 sort_order
    let max_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM ai_models WHERE provider_id = ?1",
            [&provider_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO ai_models (id, provider_id, model_id, model_name, group_name, is_default, enabled, sort_order, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, 1, ?6, ?7)",
        rusqlite::params![id, provider_id, model_id, name, group, max_order + 1, now],
    )
    .map_err(|e| format!("添加模型失败: {}", e))?;

    Ok(AiModel {
        id,
        provider_id,
        model_id,
        model_name: name,
        group_name: group,
        is_default: false,
        enabled: true,
        sort_order: max_order + 1,
        created_at: now,
    })
}

#[tauri::command]
pub fn update_model_info(
    id: String,
    model_id: Option<String>,
    model_name: Option<String>,
    group_name: Option<String>,
    db: State<Database>,
) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    let mut sets = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut param_idx = 1;

    if let Some(ref v) = model_id {
        sets.push(format!("model_id = ?{}", param_idx));
        params.push(Box::new(v.clone()));
        param_idx += 1;
    }
    if let Some(ref v) = model_name {
        sets.push(format!("model_name = ?{}", param_idx));
        params.push(Box::new(v.clone()));
        param_idx += 1;
    }
    if let Some(ref v) = group_name {
        sets.push(format!("group_name = ?{}", param_idx));
        params.push(Box::new(v.clone()));
        param_idx += 1;
    }

    if sets.is_empty() {
        return Ok(true);
    }

    let sql = format!(
        "UPDATE ai_models SET {} WHERE id = ?{}",
        sets.join(", "),
        param_idx
    );
    params.push(Box::new(id));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    conn.execute(&sql, param_refs.as_slice())
        .map_err(|e| format!("更新模型失败: {}", e))?;

    Ok(true)
}

#[tauri::command]
pub fn delete_model(id: String, db: State<Database>) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    conn.execute(
        "DELETE FROM ai_models WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| format!("删除模型失败: {}", e))?;

    Ok(true)
}

#[tauri::command]
pub fn set_default_model(id: String, db: State<Database>) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    // 先取消所有模型的默认标记
    conn.execute("UPDATE ai_models SET is_default = 0", [])
        .map_err(|e| format!("重置默认模型失败: {}", e))?;

    // 设置指定模型为默认
    conn.execute(
        "UPDATE ai_models SET is_default = 1 WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| format!("设置默认模型失败: {}", e))?;

    // 同步到旧的 system_config 表以保持向前兼容
    // 获取该模型的 model_id 和其 provider 的 api_url / api_key
    let result: Result<(String, String, String), _> = conn.query_row(
        "SELECT m.model_id, p.api_url, p.api_key
         FROM ai_models m
         JOIN ai_providers p ON m.provider_id = p.id
         WHERE m.id = ?1",
        rusqlite::params![id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    );

    if let Ok((model_id, api_url, api_key)) = result {
        let now = chrono::Local::now().to_rfc3339();
        let sync_config = |key: &str, value: &str| -> Result<(), String> {
            let uid = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO system_config (id, config_key, config_value, updated_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(config_key) DO UPDATE SET
                    config_value = excluded.config_value,
                    updated_at = excluded.updated_at",
                rusqlite::params![uid, key, value, now],
            )
            .map_err(|e| format!("同步配置失败: {}", e))?;
            Ok(())
        };

        sync_config("ai_default_model", &model_id)?;
        sync_config("ai_api_url", &api_url)?;
        sync_config("ai_api_key", &api_key)?;
    }

    Ok(true)
}

// ===== Provider 连接测试 (ISS-057) =====

#[tauri::command]
pub async fn test_provider_connection(
    provider_id: String,
    db: State<'_, Database>,
) -> Result<String, String> {
    // 从数据库读取该 provider 的配置
    let (api_url, api_key, provider_type) = {
        let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
        let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

        conn.query_row(
            "SELECT api_url, api_key, type FROM ai_providers WHERE id = ?1",
            rusqlite::params![provider_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
        )
        .map_err(|e| format!("未找到提供商: {}", e))?
    };

    if api_url.is_empty() {
        return Err("请先填写 API 地址".to_string());
    }

    // 读取全局代理设置
    let config = {
        let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
        let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
        crate::services::http_client::load_ai_config(conn)?
    };

    // 使用该 provider 的 URL 和 Key，但用全局的代理设置
    let test_config = crate::services::http_client::AiClientConfig {
        api_url: api_url.clone(),
        api_key: api_key.clone(),
        proxy_enabled: config.proxy_enabled,
        proxy_url: config.proxy_url,
        proxy_username: config.proxy_username,
        proxy_password: config.proxy_password,
        timeout_secs: config.timeout_secs,
    };

    let client = crate::services::http_client::build_client(&test_config)?;

    // 获取该 provider 下的第一个模型，如果没有模型则使用 "gpt-3.5-turbo"
    let model = {
        let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
        let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
        conn.query_row(
            "SELECT model_id FROM ai_models WHERE provider_id = ?1 ORDER BY sort_order ASC LIMIT 1",
            rusqlite::params![provider_id],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "gpt-3.5-turbo".to_string())
    };

    // 根据类型构建测试请求
    let request_body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": "Hi, this is a connection test. Reply with 'OK' only."
            }
        ],
        "stream": true,
        "max_tokens": 8192,
    });

    let mut request = client.post(&test_config.api_url);

    // 根据 provider_type 设置不同的认证头
    match provider_type.as_str() {
        "anthropic" => {
            request = request
                .header("x-api-key", &test_config.api_key)
                .header("anthropic-version", "2023-06-01");
        }
        _ => {
            // OpenAI 兼容类型 (openai / azure-openai / ollama / custom / gemini)
            if !test_config.api_key.is_empty() {
                request = request.header("Authorization", format!("Bearer {}", test_config.api_key));
            }
        }
    }
    request = request.header("Content-Type", "application/json");

    // 打印请求信息以便调试
    log::info!("测试连接 - URL: {}, Model: {}, Provider Type: {}", test_config.api_url, model, provider_type);
    log::info!("请求体: {}", request_body);
    if let Some(request_clone) = request.try_clone() {
        match request_clone.build() {
            Ok(built_request) => {
                for (k, v) in built_request.headers() {
                    log::info!("请求头 {}: {}", k, v.to_str().unwrap_or("<non-utf8>"));
                }
            }
            Err(e) => {
                log::warn!("请求头打印失败，构建请求对象出错: {}", e);
            }
        }
    } else {
        log::warn!("请求头打印失败，RequestBuilder 无法克隆");
    }
    let response = request
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    let status = response.status();
    if status.is_success() {
        Ok(format!("连接成功！模型: {}", model))
    } else {
        let body = response.text().await.unwrap_or_default();
        Err(format!("API 返回错误 ({}): {}", status.as_u16(), body))
    }
}

// ===== 旧数据迁移 (ISS-054) =====

/// 在应用启动时调用，检测旧配置并自动迁移到新的 ai_providers/ai_models 表
pub fn migrate_legacy_config(conn: &rusqlite::Connection) -> Result<(), String> {
    // 检查 ai_providers 表是否已有数据
    let provider_count: i32 = conn
        .query_row("SELECT COUNT(*) FROM ai_providers", [], |row| row.get(0))
        .unwrap_or(0);

    if provider_count > 0 {
        // 已有新表数据，无需迁移
        return Ok(());
    }

    // 检测旧配置是否存在
    let old_api_url = get_legacy_config(conn, "ai_api_url");
    let old_api_key = get_legacy_config(conn, "ai_api_key");
    let old_models_json = get_legacy_config(conn, "ai_models");
    let old_default_model = get_legacy_config(conn, "ai_default_model");

    if old_api_url.is_empty() && old_api_key.is_empty() && old_models_json.is_empty() {
        // 没有旧配置，无需迁移
        return Ok(());
    }

    log::info!("检测到旧版 AI 配置，正在自动迁移到多提供商架构...");

    let now = chrono::Local::now().to_rfc3339();
    let provider_id = uuid::Uuid::new_v4().to_string();

    // 创建默认提供商
    conn.execute(
        "INSERT INTO ai_providers (id, name, type, api_key, api_url, enabled, sort_order, created_at, updated_at)
         VALUES (?1, '默认接口', 'openai', ?2, ?3, 1, 0, ?4, ?4)",
        rusqlite::params![provider_id, old_api_key, old_api_url, now],
    )
    .map_err(|e| format!("迁移创建提供商失败: {}", e))?;

    // 解析旧模型列表
    let models: Vec<String> = if !old_models_json.is_empty() {
        serde_json::from_str(&old_models_json).unwrap_or_default()
    } else {
        Vec::new()
    };

    for (i, model_id) in models.iter().enumerate() {
        let id = uuid::Uuid::new_v4().to_string();
        let is_default = if model_id == &old_default_model { 1 } else { 0 };

        conn.execute(
            "INSERT INTO ai_models (id, provider_id, model_id, model_name, group_name, is_default, enabled, sort_order, created_at)
             VALUES (?1, ?2, ?3, ?3, '', ?4, 1, ?5, ?6)",
            rusqlite::params![id, provider_id, model_id, is_default, i as i32, now],
        )
        .map_err(|e| format!("迁移模型 {} 失败: {}", model_id, e))?;
    }

    log::info!(
        "旧配置迁移完成：创建提供商「默认接口」，迁移 {} 个模型",
        models.len()
    );

    Ok(())
}

fn get_legacy_config(conn: &rusqlite::Connection, key: &str) -> String {
    conn.query_row(
        "SELECT config_value FROM system_config WHERE config_key = ?1",
        [key],
        |row| row.get::<_, String>(0),
    )
    .unwrap_or_default()
}
