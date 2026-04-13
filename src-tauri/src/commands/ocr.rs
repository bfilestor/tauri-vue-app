use super::scope::{MemberScopeInput, ResolvedMemberScope, resolve_member_scope};
use crate::db::Database;
use crate::services::http_client;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrResult {
    pub id: String,
    pub file_id: String,
    pub record_id: String,
    pub project_id: String,
    pub checkup_date: String,
    pub raw_json: String,
    pub parsed_items: String,
    pub status: String,
    pub error_message: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrParsedItem {
    pub name: String,
    pub value: String,
    pub unit: String,
    pub reference_range: String,
    pub is_abnormal: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct OcrProgress {
    pub record_id: String,
    pub total: usize,
    pub completed: usize,
    pub current_file: String,
    pub status: String,
}

const DEFAULT_OCR_PROMPT_TEMPLATE: &str = r#"请识别图片中的医疗检查报告，提取所有检查指标。请严格按照以下JSON格式返回数组：[{"name":"指标名称","value":"数值","unit":"单位","reference_range":"参考范围","status":"正常/异常"}]。注意：reference_range字段请统一使用"reference_range"作为键名；status字段请依据数值和参考范围判断，仅返回"正常"或"异常"；如果图片中没有明确状态标记，请根据数值自行判断。只返回JSON数组，不要返回其他内容。"#;

fn load_ocr_prompt_template(conn: &rusqlite::Connection) -> String {
    conn.query_row(
        "SELECT config_value FROM system_config WHERE config_key = 'ocr_prompt_template'",
        [],
        |row| row.get(0),
    )
    .unwrap_or_else(|_| DEFAULT_OCR_PROMPT_TEMPLATE.to_string())
}

fn mask_sensitive_value(value: &str) -> String {
    let char_count = value.chars().count();
    if char_count <= 8 {
        return "***".to_string();
    }

    let start: String = value.chars().take(4).collect();
    let end: String = value
        .chars()
        .skip(char_count.saturating_sub(4))
        .take(4)
        .collect();
    format!("{}***{}", start, end)
}

fn sanitize_header_value(header_name: &str, header_value: &str) -> String {
    match header_name.to_ascii_lowercase().as_str() {
        "authorization" | "x-api-key" | "proxy-authorization" => mask_sensitive_value(header_value),
        _ => header_value.to_string(),
    }
}

fn truncate_for_log(value: &str, max_chars: usize) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= max_chars {
        return value.to_string();
    }
    let prefix: String = chars.iter().take(max_chars).collect();
    format!("{}...(truncated, total_chars={})", prefix, chars.len())
}

fn sanitize_request_body_for_log(input: &serde_json::Value) -> serde_json::Value {
    let mut sanitized = input.clone();
    sanitize_image_url_in_value(&mut sanitized);
    sanitized
}

fn sanitize_image_url_in_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(image_url_val) = map.get_mut("image_url") {
                if let serde_json::Value::Object(image_url_obj) = image_url_val {
                    if let Some(url_val) = image_url_obj.get_mut("url") {
                        if let Some(url_str) = url_val.as_str() {
                            *url_val = serde_json::Value::String(truncate_for_log(url_str, 100));
                        }
                    }
                }
            }

            for child in map.values_mut() {
                sanitize_image_url_in_value(child);
            }
        }
        serde_json::Value::Array(arr) => {
            for child in arr.iter_mut() {
                sanitize_image_url_in_value(child);
            }
        }
        _ => {}
    }
}

fn log_request_debug(
    tag: &str,
    request: &reqwest::RequestBuilder,
    request_url: &str,
    request_body: &serde_json::Value,
) {
    log::info!("[{}] Request URL: {}", tag, request_url);
    let sanitized_request_body = sanitize_request_body_for_log(request_body);
    log::info!("[{}] Request Body: {}", tag, sanitized_request_body);

    if let Some(request_clone) = request.try_clone() {
        match request_clone.build() {
            Ok(built_request) => {
                for (k, v) in built_request.headers() {
                    let key = k.as_str();
                    let raw_value = v.to_str().unwrap_or("<non-utf8>");
                    let display_value = sanitize_header_value(key, raw_value);
                    log::info!("[{}] Request Header {}: {}", tag, key, display_value);
                }
            }
            Err(e) => {
                log::warn!(
                    "[{}] Failed to build request for header logging: {}",
                    tag,
                    e
                );
            }
        }
    } else {
        log::warn!(
            "[{}] RequestBuilder cannot be cloned for header logging",
            tag
        );
    }
}

fn log_response_debug(tag: &str, status: reqwest::StatusCode, response_body: &str) {
    log::info!("[{}] Response Status: {}", tag, status);
    log::info!("[{}] Response Body: {}", tag, response_body);
}

/// 发起 OCR 识别（异步执行，通过 Event 通知前端）
#[tauri::command]
pub async fn start_ocr(
    record_id: String,
    scope: Option<MemberScopeInput>,
    app: tauri::AppHandle,
    db: tauri::State<'_, Database>,
    app_dir: tauri::State<'_, super::AppDir>,
) -> Result<String, String> {
    use tauri::Emitter;

    // 1. 查询记录和关联的文件
    let (files, checkup_date, config, model, ocr_prompt, indicators_map, scope) = {
        let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
        let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
        let scope = resolve_member_scope(conn, scope)?;

        // 获取检查日期
        let checkup_date: String = conn
            .query_row(
                "SELECT checkup_date
                 FROM checkup_records
                 WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
                rusqlite::params![&record_id, &scope.owner_user_id, &scope.member_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("记录不存在: {}", e))?;

        // 获取关联文件
        let mut stmt = conn
            .prepare(
                "SELECT f.id, f.record_id, f.project_id, f.original_filename, f.stored_path, f.mime_type, p.name
                 FROM checkup_files f
                 LEFT JOIN checkup_projects p ON f.project_id = p.id
                 WHERE f.record_id = ?1 AND f.owner_user_id = ?2 AND f.member_id = ?3
                 ORDER BY p.name ASC, f.uploaded_at ASC"
            )
            .map_err(|e| format!("查询文件失败: {}", e))?;

        let files: Vec<(String, String, String, String, String, String, String)> = stmt
            .query_map(
                rusqlite::params![&record_id, &scope.owner_user_id, &scope.member_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6).unwrap_or_default(),
                    ))
                },
            )
            .map_err(|e| format!("查询文件失败: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("解析文件数据失败: {}", e))?;

        if files.is_empty() {
            return Err("该检查记录下没有文件，请先上传检查报告图片".into());
        }

        // 获取 AI 配置
        let config = http_client::load_ai_config_for_ocr(&conn)?;
        let model = http_client::get_default_model_for_ocr(&conn);

        // 获取 OCR Prompt 模板
        let ocr_prompt = load_ocr_prompt_template(conn);

        // 加载所有项目的指标映射（用于匹配 indicator_values）
        let mut ind_stmt = conn
            .prepare(
                "SELECT id, project_id, name
                 FROM indicators
                 WHERE owner_user_id = ?1 AND member_id = ?2",
            )
            .map_err(|e| format!("查询指标失败: {}", e))?;
        let indicators: Vec<(String, String, String)> = ind_stmt
            .query_map(
                rusqlite::params![&scope.owner_user_id, &scope.member_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .map_err(|e| format!("查询指标失败: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("解析指标数据失败: {}", e))?;

        (
            files,
            checkup_date,
            config,
            model,
            ocr_prompt,
            indicators,
            scope,
        )
    };

    // 更新状态为 ocr_processing
    {
        let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
        let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
        let now = chrono::Local::now().to_rfc3339();
        conn.execute(
            "UPDATE checkup_records
             SET status = 'ocr_processing', updated_at = ?1
             WHERE id = ?2 AND owner_user_id = ?3 AND member_id = ?4",
            rusqlite::params![now, record_id, &scope.owner_user_id, &scope.member_id],
        )
        .ok();
    }

    let record_id_clone = record_id.clone();
    let app_dir_path = app_dir.0.clone();
    let total_files = files.len();

    // 2. 异步执行 OCR
    tokio::spawn(async move {
        let client = match http_client::build_client(&config) {
            Ok(c) => c,
            Err(e) => {
                log::error!("OCR 创建客户端失败: {}", e);
                app.emit(
                    "ocr_error",
                    serde_json::json!({
                        "record_id": record_id_clone,
                        "error": e,
                    }),
                )
                .ok();
                return;
            }
        };

        let mut success_count = 0;
        let mut error_messages = Vec::new();

        for (i, (file_id, _rec_id, project_id, filename, stored_path, mime_type, _project_name)) in
            files.iter().enumerate()
        {
            // 发送进度事件
            app.emit(
                "ocr_progress",
                OcrProgress {
                    record_id: record_id_clone.clone(),
                    total: total_files,
                    completed: i,
                    current_file: filename.clone(),
                    status: "processing".to_string(),
                },
            )
            .ok();

            // 避免请求过快导致 Rate Limit，添加延时 (如果是第一个文件不需要延时)
            if i > 0 {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }

            log::info!("正在处理文件 {}/{}: {}", i + 1, total_files, filename);

            // 读取图片文件并转 Base64
            let full_path = app_dir_path.join(stored_path);
            let file_bytes = match std::fs::read(&full_path) {
                Ok(b) => b,
                Err(e) => {
                    error_messages.push(format!("{}: 读取文件失败 - {}", filename, e));
                    save_ocr_error(
                        &app,
                        &record_id_clone,
                        file_id,
                        &project_id,
                        &checkup_date,
                        &scope.owner_user_id,
                        &scope.member_id,
                        &format!("读取文件失败: {}", e),
                    );
                    continue;
                }
            };

            let b64 =
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &file_bytes);
            let image_data_url = format!("data:{};base64,{}", mime_type, b64);

            // 构建视觉 API 请求
            let request_body = serde_json::json!({
                "model": model,
                "messages": [
                    {
                        "role": "user",
                        "content": [
                            {
                                "type": "text",
                                "text": ocr_prompt
                            },
                            {
                                "type": "image_url",
                                "image_url": {
                                    "url": image_data_url
                                }
                            }
                        ]
                    }
                ],
                "stream": true,
                "max_tokens": 4096,
            });

            // 发送请求
            log::info!(
                "OCR 场景路由 - scene: ocr, model: {}, url: {}",
                model, config.api_url
            );
            let request = client
                .post(&config.api_url)
                .header("Authorization", format!("Bearer {}", config.api_key))
                .header("Content-Type", "application/json");

            log_request_debug("start_ocr", &request, &config.api_url, &request_body);

            let response = match request.json(&request_body).send().await {
                Ok(r) => r,
                Err(e) => {
                    let err_msg = format!("{}: 请求失败 - {}", filename, e);
                    error_messages.push(err_msg.clone());
                    save_ocr_error(
                        &app,
                        &record_id_clone,
                        file_id,
                        &project_id,
                        &checkup_date,
                        &scope.owner_user_id,
                        &scope.member_id,
                        &err_msg,
                    );
                    continue;
                }
            };

            let status = response.status();
            let response_body = response.text().await.unwrap_or_default();
            log_response_debug("start_ocr", status, &response_body);

            if !status.is_success() {
                let err_msg = format!("{}: API错误({}) - {}", filename, status, response_body);
                error_messages.push(err_msg.clone());
                save_ocr_error(
                    &app,
                    &record_id_clone,
                    file_id,
                    &project_id,
                    &checkup_date,
                    &scope.owner_user_id,
                    &scope.member_id,
                    &err_msg,
                );
                continue;
            }

            // 解析响应（兼容 stream=true 的 SSE 格式）
            let content = match extract_ai_content_from_response_body(&response_body) {
                Ok(v) => v,
                Err(e) => {
                    let err_msg = format!("{}: 解析响应失败 - {}", filename, e);
                    error_messages.push(err_msg.clone());
                    save_ocr_error(
                        &app,
                        &record_id_clone,
                        file_id,
                        &project_id,
                        &checkup_date,
                        &scope.owner_user_id,
                        &scope.member_id,
                        &err_msg,
                    );
                    continue;
                }
            };

            log::info!("OCR AI Raw Response (File: {}): {}", filename, content);

            // 尝试解析 JSON 数组
            let parsed_items = extract_json_array(&content);
            let parsed_items_str = serde_json::to_string(&parsed_items).unwrap_or("[]".to_string());

            // 保存 OCR 结果到数据库
            let ocr_id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Local::now().to_rfc3339();

            // 获取数据库连接并保存
            if let Some(db_state) = app.try_state::<Database>() {
                if let Ok(conn_guard) = db_state.conn.lock() {
                    if let Some(conn) = conn_guard.as_ref() {
                        let _: Result<usize, _> = conn.execute(
                        "INSERT INTO ocr_results (id, owner_user_id, member_id, file_id, record_id, project_id, checkup_date, raw_json, parsed_items, status, error_message, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'success', '', ?10)",
                        rusqlite::params![
                            ocr_id,
                            &scope.owner_user_id,
                            &scope.member_id,
                            file_id,
                            record_id_clone,
                            project_id,
                            checkup_date,
                            content,
                            parsed_items_str,
                            now
                        ],
                    );

                        // 清理旧记录
                        let _: Result<usize, _> = conn.execute(
                        "DELETE FROM indicator_values
                         WHERE ocr_result_id IN (
                            SELECT id FROM ocr_results
                            WHERE file_id = ?1 AND owner_user_id = ?2 AND member_id = ?3 AND id != ?4
                         )",
                        rusqlite::params![&file_id, &scope.owner_user_id, &scope.member_id, &ocr_id],
                    );
                        let _: Result<usize, _> = conn.execute(
                            "DELETE FROM ocr_results
                             WHERE file_id = ?1 AND owner_user_id = ?2 AND member_id = ?3 AND id != ?4",
                            rusqlite::params![&file_id, &scope.owner_user_id, &scope.member_id, &ocr_id],
                        );

                        // 将解析出的指标值写入 indicator_values
                        for item in &parsed_items {
                            // 尝试匹配指标定义
                            let indicator_match = indicators_map.iter().find(|(_, pid, name)| {
                                pid == project_id && name_fuzzy_match(name, &item.name)
                            });

                            if let Some((indicator_id, _, _)) = indicator_match {
                                let value: Option<f64> = item.value.parse().ok();
                                let iv_id = uuid::Uuid::new_v4().to_string();
                                let _: Result<usize, _> = conn.execute(
                                "INSERT INTO indicator_values (id, owner_user_id, member_id, ocr_result_id, record_id, project_id, indicator_id, checkup_date, value, value_text, is_abnormal, created_at)
                                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                                rusqlite::params![
                                    iv_id,
                                    &scope.owner_user_id,
                                    &scope.member_id,
                                    ocr_id,
                                    record_id_clone,
                                    project_id,
                                    indicator_id,
                                    checkup_date,
                                    value,
                                    item.value,
                                    item.is_abnormal as i32,
                                    now
                                ],
                            );
                            }
                        }
                    }
                }
            }

            success_count += 1;
        }

        // 更新检查记录状态
        if let Some(db_state) = app.try_state::<Database>() {
            if let Ok(conn_guard) = db_state.conn.lock() {
                if let Some(conn) = conn_guard.as_ref() {
                    let now = chrono::Local::now().to_rfc3339();
                    let new_status = if success_count > 0 {
                        "ocr_done"
                    } else {
                        "pending_ocr"
                    };
                    let _: Result<usize, _> = conn.execute(
                        "UPDATE checkup_records
                         SET status = ?1, updated_at = ?2
                         WHERE id = ?3 AND owner_user_id = ?4 AND member_id = ?5",
                        rusqlite::params![
                            new_status,
                            now,
                            record_id_clone,
                            &scope.owner_user_id,
                            &scope.member_id
                        ],
                    );
                }
            }
        }

        // 发送完成事件
        app.emit(
            "ocr_complete",
            serde_json::json!({
                "record_id": record_id_clone,
                "total": total_files,
                "success": success_count,
                "errors": error_messages,
            }),
        )
        .ok();
    });

    Ok(record_id)
}

/// 重试单个 OCR 任务 (创建新记录，保留历史)
#[tauri::command]
pub async fn retry_ocr(
    ocr_id: String,
    scope: Option<MemberScopeInput>,
    app: tauri::AppHandle,
    db: tauri::State<'_, Database>,
    app_dir: tauri::State<'_, super::AppDir>,
) -> Result<String, String> {
    use tauri::Emitter;

    // 1. 查询必要信息
    let (file_info, record_id, checkup_date, config, model, ocr_prompt, indicators_map, scope) = {
        let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
        let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
        let scope = resolve_member_scope(conn, scope)?;

        // 查询 OCR 记录关联的信息
        let (file_id, record_id, project_id) = conn
            .query_row(
                "SELECT file_id, record_id, project_id
                     FROM ocr_results
                     WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
                rusqlite::params![&ocr_id, &scope.owner_user_id, &scope.member_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .map_err(|e| format!("OCR记录不存在: {}", e))?;

        // 查询日期
        let checkup_date: String = conn
            .query_row(
                "SELECT checkup_date
                     FROM checkup_records
                     WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
                rusqlite::params![&record_id, &scope.owner_user_id, &scope.member_id],
                |row| row.get(0),
            )
            .unwrap_or_default();

        // 查询文件信息
        let (filename, stored_path, mime_type, project_name) = conn
            .query_row(
                "SELECT f.original_filename, f.stored_path, f.mime_type, p.name
             FROM checkup_files f
             LEFT JOIN checkup_projects p ON f.project_id = p.id
             WHERE f.id = ?1 AND f.owner_user_id = ?2 AND f.member_id = ?3",
                rusqlite::params![&file_id, &scope.owner_user_id, &scope.member_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3).unwrap_or_default(),
                    ))
                },
            )
            .map_err(|e| format!("文件不存在: {}", e))?;

        let config = http_client::load_ai_config_for_ocr(&conn)?;
        let model = http_client::get_default_model_for_ocr(&conn);
        let ocr_prompt = load_ocr_prompt_template(conn);

        // 加载指标映射
        let mut ind_stmt = conn
            .prepare(
                "SELECT id, project_id, name
                     FROM indicators
                     WHERE owner_user_id = ?1 AND member_id = ?2",
            )
            .unwrap();
        let indicators: Vec<(String, String, String)> = ind_stmt
            .query_map(
                rusqlite::params![&scope.owner_user_id, &scope.member_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        (
            (
                file_id,
                project_id,
                filename,
                stored_path,
                mime_type,
                project_name,
            ),
            record_id,
            checkup_date,
            config,
            model,
            ocr_prompt,
            indicators,
            scope,
        )
    };

    let (file_id, project_id, filename, stored_path, mime_type, _project_name) = file_info;
    let new_ocr_id = uuid::Uuid::new_v4().to_string(); // 使用新 ID
    let ocr_id_clone = new_ocr_id.clone(); // 这里的变量名保持一致方便后面闭包使用
    let record_id_clone = record_id.clone();
    let app_dir_path = app_dir.0.clone();

    // 插入新记录 (状态 processing)
    if let Some(db_state) = app.try_state::<Database>() {
        if let Ok(conn_guard) = db_state.conn.lock() {
            if let Some(conn) = conn_guard.as_ref() {
                let now = chrono::Local::now().to_rfc3339();
                let _: Result<usize, _> = conn.execute(
                 "INSERT INTO ocr_results (id, owner_user_id, member_id, file_id, record_id, project_id, checkup_date, raw_json, parsed_items, status, error_message, created_at)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, '', '[]', 'processing', '', ?8)",
                 rusqlite::params![
                    ocr_id_clone,
                    &scope.owner_user_id,
                    &scope.member_id,
                    file_id,
                    record_id,
                    project_id,
                    checkup_date,
                    now
                 ],
             );

                // 清理该文件的旧 OCR 记录 (只保留当前新创建的)
                // 1. 删除旧记录关联的指标值
                let _: Result<usize, _> = conn.execute(
                    "DELETE FROM indicator_values
                  WHERE ocr_result_id IN (
                    SELECT id FROM ocr_results
                    WHERE file_id = ?1 AND owner_user_id = ?2 AND member_id = ?3 AND id != ?4
                  )",
                    rusqlite::params![
                        &file_id,
                        &scope.owner_user_id,
                        &scope.member_id,
                        &ocr_id_clone
                    ],
                );
                // 2. 删除旧 OCR 记录
                let _: Result<usize, _> = conn.execute(
                    "DELETE FROM ocr_results
                     WHERE file_id = ?1 AND owner_user_id = ?2 AND member_id = ?3 AND id != ?4",
                    rusqlite::params![
                        &file_id,
                        &scope.owner_user_id,
                        &scope.member_id,
                        &ocr_id_clone
                    ],
                );
            }
        }
    }

    // 发送开始事件
    app.emit(
        "ocr_progress",
        OcrProgress {
            record_id: record_id.clone(),
            total: 1,
            completed: 0,
            current_file: filename.clone(),
            status: "processing".to_string(),
        },
    )
    .ok();

    tokio::spawn(async move {
        let client = match http_client::build_client(&config) {
            Ok(c) => c,
            Err(e) => {
                update_ocr_failed(&app, &ocr_id_clone, &format!("创建客户端失败: {}", e));
                return;
            }
        };

        // 读取文件
        let full_path = app_dir_path.join(stored_path);
        let file_bytes = match std::fs::read(&full_path) {
            Ok(b) => b,
            Err(e) => {
                update_ocr_failed(&app, &ocr_id_clone, &format!("读取文件失败: {}", e));
                return;
            }
        };

        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &file_bytes);
        let image_data_url = format!("data:{};base64,{}", mime_type, b64);

        let request_body = serde_json::json!({
            "model": model,
            "messages": [
                { "role": "user", "content": [
                    { "type": "text", "text": ocr_prompt },
                    { "type": "image_url", "image_url": { "url": image_data_url } }
                ]}
            ],
            "stream": true,
            "max_tokens": 4096,
        });

        log::info!(
            "OCR 场景路由 - scene: ocr, model: {}, url: {}",
            model, config.api_url
        );
        let request = client
            .post(&config.api_url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json");

        log_request_debug("retry_ocr", &request, &config.api_url, &request_body);

        let response = match request.json(&request_body).send().await {
            Ok(r) => r,
            Err(e) => {
                update_ocr_failed(&app, &ocr_id_clone, &format!("请求失败: {}", e));
                return;
            }
        };

        let status = response.status();
        let response_body = response.text().await.unwrap_or_default();
        log_response_debug("retry_ocr", status, &response_body);

        if !status.is_success() {
            update_ocr_failed(
                &app,
                &ocr_id_clone,
                &format!("API错误({}): {}", status, response_body),
            );
            return;
        }

        let content = match extract_ai_content_from_response_body(&response_body) {
            Ok(v) => v,
            Err(e) => {
                update_ocr_failed(&app, &ocr_id_clone, &format!("解析响应失败: {}", e));
                return;
            }
        };
        log::info!("Retry OCR AI Raw Response: {}", content);
        let parsed_items = extract_json_array(&content);
        let parsed_items_str = serde_json::to_string(&parsed_items).unwrap_or("[]".to_string());

        // 更新数据库
        if let Some(db_state) = app.try_state::<Database>() {
            if let Ok(conn_guard) = db_state.conn.lock() {
                if let Some(conn) = conn_guard.as_ref() {
                    let now = chrono::Local::now().to_rfc3339();

                    // 更新 ocr_results
                    let _: Result<usize, _> = conn.execute(
                    "UPDATE ocr_results SET status = 'success', raw_json = ?1, parsed_items = ?2, error_message = '', created_at = ?3 WHERE id = ?4",
                    rusqlite::params![content, parsed_items_str, now, ocr_id_clone],
                );

                    // 写入 indicator_values (新记录不需要删除旧的)

                    for item in &parsed_items {
                        let indicator_match = indicators_map.iter().find(|(_, pid, name)| {
                            pid == &project_id && name_fuzzy_match(name, &item.name)
                        });

                        if let Some((indicator_id, _, _)) = indicator_match {
                            let value: Option<f64> = item.value.parse().ok();
                            let iv_id = uuid::Uuid::new_v4().to_string();
                            let _: Result<usize, _> = conn.execute(
                             "INSERT INTO indicator_values (id, owner_user_id, member_id, ocr_result_id, record_id, project_id, indicator_id, checkup_date, value, value_text, is_abnormal, created_at)
                              VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                             rusqlite::params![
                                iv_id,
                                &scope.owner_user_id,
                                &scope.member_id,
                                ocr_id_clone,
                                record_id_clone,
                                project_id,
                                indicator_id,
                                checkup_date,
                                value,
                                item.value,
                                item.is_abnormal as i32,
                                now
                             ],
                         );
                        }
                    }
                }
            }
        }

        // 发送完成事件
        app.emit(
            "ocr_complete",
            serde_json::json!({
                "record_id": record_id_clone,
                "total": 1,
                "success": 1,
                "errors": Vec::<String>::new()
            }),
        )
        .ok();
    });

    Ok("Retry started".to_string())
}

fn update_ocr_failed(app: &tauri::AppHandle, ocr_id: &str, error: &str) {
    if let Some(db_state) = app.try_state::<Database>() {
        if let Ok(conn_guard) = db_state.conn.lock() {
            if let Some(conn) = conn_guard.as_ref() {
                let _: Result<usize, _> = conn.execute(
                    "UPDATE ocr_results SET status = 'failed', error_message = ?1 WHERE id = ?2",
                    [error, ocr_id],
                );
            }
        }
    }
}

/// 保存 OCR 错误结果
fn save_ocr_error(
    app: &tauri::AppHandle,
    record_id: &str,
    file_id: &str,
    project_id: &str,
    checkup_date: &str,
    owner_user_id: &str,
    member_id: &str,
    error_msg: &str,
) {
    if let Some(db_state) = app.try_state::<Database>() {
        if let Ok(conn_guard) = db_state.conn.lock() {
            if let Some(conn) = conn_guard.as_ref() {
                let ocr_id = uuid::Uuid::new_v4().to_string();
                let now = chrono::Local::now().to_rfc3339();
                let _: Result<usize, _> = conn.execute(
                "INSERT INTO ocr_results (id, owner_user_id, member_id, file_id, record_id, project_id, checkup_date, raw_json, parsed_items, status, error_message, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, '', '[]', 'failed', ?8, ?9)",
                rusqlite::params![
                    ocr_id,
                    owner_user_id,
                    member_id,
                    file_id,
                    record_id,
                    project_id,
                    checkup_date,
                    error_msg,
                    now
                ],
            );

                // 清理旧记录
                let _: Result<usize, _> = conn.execute(
                    "DELETE FROM indicator_values
                 WHERE ocr_result_id IN (
                    SELECT id FROM ocr_results
                    WHERE file_id = ?1 AND owner_user_id = ?2 AND member_id = ?3 AND id != ?4
                 )",
                    rusqlite::params![file_id, owner_user_id, member_id, &ocr_id],
                );
                let _: Result<usize, _> = conn.execute(
                    "DELETE FROM ocr_results
                     WHERE file_id = ?1 AND owner_user_id = ?2 AND member_id = ?3 AND id != ?4",
                    rusqlite::params![file_id, owner_user_id, member_id, &ocr_id],
                );
            }
        }
    }
}

/// 更新单个 OCR 指标项
fn update_ocr_item_with_conn(
    conn: &Connection,
    ocr_id: String,
    index: usize,
    item: OcrParsedItem,
    scope: &ResolvedMemberScope,
) -> Result<(), String> {
    // 1. 获取当前数据
    let (parsed_items_str, project_id, record_id, checkup_date): (String, String, String, String) =
        conn.query_row(
            "SELECT parsed_items, project_id, record_id, checkup_date
         FROM ocr_results
         WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
            rusqlite::params![&ocr_id, &scope.owner_user_id, &scope.member_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|e| format!("OCR记录不存在: {}", e))?;

    let mut parsed_items: Vec<OcrParsedItem> =
        serde_json::from_str(&parsed_items_str).map_err(|e| format!("解析JSON失败: {}", e))?;

    if index >= parsed_items.len() {
        return Err("索引超出范围".into());
    }

    // 2. 更新数据
    parsed_items[index] = item;
    let new_json = serde_json::to_string(&parsed_items).unwrap();

    // 3. 更新 ocr_results
    let now = chrono::Local::now().to_rfc3339();
    conn.execute(
        "UPDATE ocr_results SET parsed_items = ?1 WHERE id = ?2",
        [&new_json, &ocr_id],
    )
    .map_err(|e| format!("更新失败: {}", e))?;

    // 4. 重新生成 indicator_values
    // 加载指标映射
    let mut ind_stmt = conn
        .prepare(
            "SELECT id, project_id, name
             FROM indicators
             WHERE owner_user_id = ?1 AND member_id = ?2",
        )
        .unwrap();
    let indicators_map: Vec<(String, String, String)> = ind_stmt
        .query_map(
            rusqlite::params![&scope.owner_user_id, &scope.member_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    // 先删除旧的
    conn.execute(
        "DELETE FROM indicator_values WHERE ocr_result_id = ?1",
        [&ocr_id],
    )
    .map_err(|e| format!("清理旧数据失败: {}", e))?;

    // 写入新的
    for item in &parsed_items {
        let indicator_match = indicators_map
            .iter()
            .find(|(_, pid, name)| pid == &project_id && name_fuzzy_match(name, &item.name));

        if let Some((indicator_id, _, _)) = indicator_match {
            let value: Option<f64> = item.value.parse().ok();
            let iv_id = uuid::Uuid::new_v4().to_string();
            let _: Result<usize, _> = conn.execute(
                 "INSERT INTO indicator_values (id, owner_user_id, member_id, ocr_result_id, record_id, project_id, indicator_id, checkup_date, value, value_text, is_abnormal, created_at)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                 rusqlite::params![
                    iv_id,
                    &scope.owner_user_id,
                    &scope.member_id,
                    ocr_id,
                    record_id,
                    project_id,
                    indicator_id,
                    checkup_date,
                    value,
                    item.value,
                    item.is_abnormal as i32,
                    now
                 ],
             );
        }
    }

    Ok(())
}

/// 更新单个 OCR 指标项
#[tauri::command]
pub fn update_ocr_item(
    ocr_id: String,
    index: usize,
    item: OcrParsedItem,
    scope: Option<MemberScopeInput>,
    db: tauri::State<Database>,
) -> Result<(), String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    update_ocr_item_with_conn(conn, ocr_id, index, item, &scope)
}

/// 查询 OCR 状态
fn get_ocr_status_with_conn(
    conn: &Connection,
    record_id: String,
    scope: &ResolvedMemberScope,
) -> Result<serde_json::Value, String> {
    let total_files: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM checkup_files WHERE record_id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
            rusqlite::params![&record_id, &scope.owner_user_id, &scope.member_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    let total_ocr: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM ocr_results WHERE record_id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
            rusqlite::params![&record_id, &scope.owner_user_id, &scope.member_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    let success_ocr: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM ocr_results WHERE record_id = ?1 AND owner_user_id = ?2 AND member_id = ?3 AND status = 'success'",
            rusqlite::params![&record_id, &scope.owner_user_id, &scope.member_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    let failed_ocr: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM ocr_results WHERE record_id = ?1 AND owner_user_id = ?2 AND member_id = ?3 AND status = 'failed'",
            rusqlite::params![&record_id, &scope.owner_user_id, &scope.member_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    let record_status: String = conn
        .query_row(
            "SELECT status FROM checkup_records WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
            rusqlite::params![&record_id, &scope.owner_user_id, &scope.member_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("记录不存在: {}", e))?;

    Ok(serde_json::json!({
        "record_status": record_status,
        "total_files": total_files,
        "total_ocr": total_ocr,
        "success_ocr": success_ocr,
        "failed_ocr": failed_ocr,
    }))
}

/// 查询 OCR 状态
#[tauri::command]
pub fn get_ocr_status(
    record_id: String,
    scope: Option<MemberScopeInput>,
    db: tauri::State<Database>,
) -> Result<serde_json::Value, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    get_ocr_status_with_conn(conn, record_id, &scope)
}

/// 获取 OCR 结果
fn get_ocr_results_with_conn(
    conn: &Connection,
    record_id: String,
    scope: &ResolvedMemberScope,
) -> Result<Vec<OcrResult>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, file_id, record_id, project_id, checkup_date, raw_json, parsed_items, status, error_message, created_at
             FROM ocr_results WHERE record_id = ?1 AND owner_user_id = ?2 AND member_id = ?3
             ORDER BY created_at ASC"
        )
        .map_err(|e| format!("查询OCR结果失败: {}", e))?;

    let results = stmt
        .query_map(
            rusqlite::params![&record_id, &scope.owner_user_id, &scope.member_id],
            |row| {
                Ok(OcrResult {
                    id: row.get(0)?,
                    file_id: row.get(1)?,
                    record_id: row.get(2)?,
                    project_id: row.get(3)?,
                    checkup_date: row.get(4)?,
                    raw_json: row.get(5)?,
                    parsed_items: row.get(6)?,
                    status: row.get(7)?,
                    error_message: row.get(8)?,
                    created_at: row.get(9)?,
                })
            },
        )
        .map_err(|e| format!("查询失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析数据失败: {}", e))?;

    Ok(results)
}

/// 获取 OCR 结果
#[tauri::command]
pub fn get_ocr_results(
    record_id: String,
    scope: Option<MemberScopeInput>,
    db: tauri::State<Database>,
) -> Result<Vec<OcrResult>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    get_ocr_results_with_conn(conn, record_id, &scope)
}

/// 从 AI 返回的文本中提取 JSON 数组
fn extract_json_array(content: &str) -> Vec<OcrParsedItem> {
    // 尝试直接解析
    if let Ok(items) = serde_json::from_str::<Vec<OcrParsedItem>>(content) {
        return items;
    }

    // 尝试从 markdown code block 中提取
    let trimmed = content.trim();
    let json_str = if trimmed.starts_with("```json") {
        trimmed
            .strip_prefix("```json")
            .and_then(|s| s.strip_suffix("```"))
            .unwrap_or(trimmed)
            .trim()
    } else if trimmed.starts_with("```") {
        trimmed
            .strip_prefix("```")
            .and_then(|s| s.strip_suffix("```"))
            .unwrap_or(trimmed)
            .trim()
    } else {
        trimmed
    };

    // 尝试找到 [ 和 ] 之间的内容
    if let Some(start) = json_str.find('[') {
        if let Some(end) = json_str.rfind(']') {
            let array_str = &json_str[start..=end];
            if let Ok(items) = serde_json::from_str::<Vec<OcrParsedItem>>(array_str) {
                return items;
            }
            // 尝试更宽松的解析：先解析为 Value 数组
            if let Ok(values) = serde_json::from_str::<Vec<serde_json::Value>>(array_str) {
                return values
                    .iter()
                    .filter_map(|v| {
                        Some(OcrParsedItem {
                            name: v
                                .get("name")
                                .or(v.get("指标名称"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            value: v
                                .get("value")
                                .or(v.get("数值"))
                                .and_then(|v| match v {
                                    serde_json::Value::String(s) => Some(s.clone()),
                                    serde_json::Value::Number(n) => Some(n.to_string()),
                                    _ => None,
                                })
                                .unwrap_or_default(),
                            unit: v
                                .get("unit")
                                .or(v.get("单位"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            reference_range: v
                                .get("reference_range")
                                .or(v.get("参考范围"))
                                .or(v.get("range"))
                                .or(v.get("参考值"))
                                .or(v.get("reference"))
                                .map(|v| match v {
                                    serde_json::Value::String(s) => s.clone(),
                                    serde_json::Value::Number(n) => n.to_string(),
                                    serde_json::Value::Array(a) => a
                                        .iter()
                                        .map(|x| x.as_str().unwrap_or("").to_string())
                                        .collect::<Vec<_>>()
                                        .join("-"),
                                    _ => "".to_string(),
                                })
                                .unwrap_or("".to_string()),
                            is_abnormal: v
                                .get("is_abnormal")
                                .or(v.get("是否异常"))
                                .or(v.get("abnormal"))
                                .or(v.get("status"))
                                .or(v.get("状态"))
                                .map(|val| {
                                    if let Some(b) = val.as_bool() {
                                        return b;
                                    }
                                    if let Some(s) = val.as_str() {
                                        let s = s.trim().to_lowercase();
                                        return s == "true"
                                            || s == "yes"
                                            || s == "1"
                                            || s == "异常"
                                            || s == "是"
                                            || s == "high"
                                            || s == "low"
                                            || s.contains("↑")
                                            || s.contains("↓")
                                            || s.contains("+");
                                    }
                                    false
                                })
                                .unwrap_or(false),
                        })
                    })
                    .collect();
            }
        }
    }

    Vec::new()
}

/// 从 AI 接口响应体中提取最终文本内容，兼容：
/// 1) 普通 JSON: choices[0].message.content
/// 2) SSE 流: data: {...choices[0].delta.content...}
fn extract_ai_content_from_response_body(response_body: &str) -> Result<String, String> {
    if let Ok(resp_json) = serde_json::from_str::<serde_json::Value>(response_body) {
        if let Some(content) = extract_content_from_chat_json(&resp_json) {
            return Ok(content);
        }
        return Err("JSON 响应缺少可用的 content 字段".to_string());
    }

    if let Some(content) = extract_content_from_sse_body(response_body) {
        return Ok(content);
    }

    Err("响应既不是标准 JSON，也不是可解析的 SSE 数据".to_string())
}

fn extract_content_from_chat_json(v: &serde_json::Value) -> Option<String> {
    let content_node = v
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"));

    if let Some(s) = content_node.and_then(|c| c.as_str()) {
        return Some(s.to_string());
    }

    // 兼容 content 为数组片段的返回
    if let Some(arr) = content_node.and_then(|c| c.as_array()) {
        let text = arr
            .iter()
            .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
            .collect::<String>();
        if !text.is_empty() {
            return Some(text);
        }
    }

    // 兼容部分接口直接返回 output_text
    v.get("output_text")
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
}

fn extract_content_from_sse_body(response_body: &str) -> Option<String> {
    let mut merged = String::new();

    for raw_line in response_body.lines() {
        let line = raw_line.trim();
        if !line.starts_with("data:") {
            continue;
        }

        let payload = line.trim_start_matches("data:").trim();
        if payload.is_empty() {
            continue;
        }
        if payload == "[DONE]" {
            break;
        }

        let chunk = match serde_json::from_str::<serde_json::Value>(payload) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(delta_text) = chunk
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("delta"))
            .and_then(|d| d.get("content"))
            .and_then(|c| c.as_str())
        {
            merged.push_str(delta_text);
            continue;
        }

        // 兼容流式最后一个分片直接给出完整 message.content
        if let Some(msg_text) = extract_content_from_chat_json(&chunk) {
            merged.push_str(&msg_text);
        }
    }

    if merged.is_empty() {
        None
    } else {
        Some(merged)
    }
}

/// 模糊匹配指标名称
fn name_fuzzy_match(indicator_name: &str, ocr_name: &str) -> bool {
    let a = indicator_name.trim().to_lowercase();
    let b = ocr_name.trim().to_lowercase();

    if a == b {
        return true;
    }

    // 包含匹配
    if a.contains(&b) || b.contains(&a) {
        return true;
    }

    // 去掉括号内容后匹配
    let strip_parens = |s: &str| -> String {
        let mut result = String::new();
        let mut depth = 0;
        for ch in s.chars() {
            match ch {
                '(' | '（' => depth += 1,
                ')' | '）' => {
                    depth -= 1;
                }
                _ if depth == 0 => result.push(ch),
                _ => {}
            }
        }
        result.trim().to_string()
    };

    let a_stripped = strip_parens(&a);
    let b_stripped = strip_parens(&b);

    a_stripped == b_stripped || a_stripped.contains(&b_stripped) || b_stripped.contains(&a_stripped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_database(name: &str) -> (Database, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "health-monitor-ocr-tests-{}-{}",
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

    fn seed_base_schema(conn: &Connection) {
        conn.execute(
            "INSERT INTO checkup_projects (
                id, owner_user_id, member_id, name, description, sort_order, is_active, created_at, updated_at
             ) VALUES ('proj-blood', 'user-1', 'member-a', '血常规', '', 0, 1, '2026-04-08T00:00:00+08:00', '2026-04-08T00:00:00+08:00')",
            [],
        )
        .expect("project should seed");
        conn.execute(
            "INSERT INTO indicators (
                id, project_id, owner_user_id, member_id, name, unit, reference_range, sort_order, is_core, created_at
             ) VALUES ('ind-wbc', 'proj-blood', 'user-1', 'member-a', '白细胞', '10^9/L', '3.5-9.5', 0, 1, '2026-04-08T00:00:00+08:00')",
            [],
        )
        .expect("indicator should seed");
    }

    fn seed_member_bundle(
        conn: &Connection,
        owner_user_id: &str,
        member_id: &str,
        record_id: &str,
        file_id: &str,
        ocr_id: &str,
        parsed_items: &str,
        status: &str,
    ) {
        conn.execute(
            "INSERT INTO checkup_records (
                id, owner_user_id, member_id, member_name_snapshot, checkup_date, status, notes, created_at, updated_at
             ) VALUES (?1, ?2, ?3, '成员', '2026-04-08', 'ocr_done', '', '2026-04-08T00:00:00+08:00', '2026-04-08T00:00:00+08:00')",
            rusqlite::params![record_id, owner_user_id, member_id],
        )
        .expect("record should seed");
        conn.execute(
            "INSERT INTO checkup_files (
                id, owner_user_id, member_id, record_id, project_id, original_filename, stored_path, file_size, mime_type, uploaded_at
             ) VALUES (?1, ?2, ?3, ?4, 'proj-blood', 'report.png', 'pictures/report.png', 100, 'image/png', '2026-04-08T00:00:00+08:00')",
            rusqlite::params![file_id, owner_user_id, member_id, record_id],
        )
        .expect("file should seed");
        conn.execute(
            "INSERT INTO ocr_results (
                id, owner_user_id, member_id, file_id, record_id, project_id, checkup_date, raw_json, parsed_items, status, error_message, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, 'proj-blood', '2026-04-08', '{}', ?6, ?7, '', '2026-04-08T00:00:00+08:00')",
            rusqlite::params![ocr_id, owner_user_id, member_id, file_id, record_id, parsed_items, status],
        )
        .expect("ocr should seed");
    }

    #[test]
    fn get_ocr_status_only_counts_current_member_rows() {
        let (db, dir) = create_test_database("status");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");
            seed_base_schema(conn);
            seed_member_bundle(
                conn, "user-1", "member-a", "record-a", "file-a", "ocr-a", "[]", "success",
            );
            seed_member_bundle(
                conn, "user-1", "member-b", "record-b", "file-b", "ocr-b", "[]", "failed",
            );

            let status = get_ocr_status_with_conn(
                conn,
                "record-a".to_string(),
                &member_scope("user-1", "member-a", "本人"),
            )
            .expect("status should load");

            assert_eq!(status["total_files"].as_i64(), Some(1));
            assert_eq!(status["total_ocr"].as_i64(), Some(1));
            assert_eq!(status["success_ocr"].as_i64(), Some(1));
            assert_eq!(status["failed_ocr"].as_i64(), Some(0));
        }
        cleanup_test_database(&db, dir);
    }

    #[test]
    fn get_ocr_results_only_returns_current_member_rows() {
        let (db, dir) = create_test_database("results");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");
            seed_base_schema(conn);
            seed_member_bundle(
                conn,
                "user-1",
                "member-a",
                "record-a",
                "file-a",
                "ocr-a",
                r#"[{"name":"白细胞","value":"11.2","unit":"10^9/L","reference_range":"3.5-9.5","is_abnormal":true}]"#,
                "success",
            );
            seed_member_bundle(
                conn,
                "user-1",
                "member-b",
                "record-b",
                "file-b",
                "ocr-b",
                r#"[{"name":"白细胞","value":"15.0","unit":"10^9/L","reference_range":"3.5-9.5","is_abnormal":true}]"#,
                "success",
            );

            let results = get_ocr_results_with_conn(
                conn,
                "record-a".to_string(),
                &member_scope("user-1", "member-a", "本人"),
            )
            .expect("results should load");

            assert_eq!(results.len(), 1);
            assert_eq!(results[0].id, "ocr-a");
            assert!(results[0].parsed_items.contains("11.2"));
        }
        cleanup_test_database(&db, dir);
    }

    #[test]
    fn update_ocr_item_only_rebuilds_indicator_values_inside_current_member_scope() {
        let (db, dir) = create_test_database("update-item");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");
            seed_base_schema(conn);
            seed_member_bundle(
                conn,
                "user-1",
                "member-a",
                "record-a",
                "file-a",
                "ocr-a",
                r#"[{"name":"白细胞","value":"11.2","unit":"10^9/L","reference_range":"3.5-9.5","is_abnormal":true}]"#,
                "success",
            );
            seed_member_bundle(
                conn,
                "user-1",
                "member-b",
                "record-b",
                "file-b",
                "ocr-b",
                r#"[{"name":"白细胞","value":"15.0","unit":"10^9/L","reference_range":"3.5-9.5","is_abnormal":true}]"#,
                "success",
            );
            conn.execute(
                "INSERT INTO indicator_values (
                    id, ocr_result_id, record_id, project_id, indicator_id, owner_user_id, member_id, checkup_date, value, value_text, is_abnormal, created_at
                 ) VALUES
                    ('iv-a', 'ocr-a', 'record-a', 'proj-blood', 'ind-wbc', 'user-1', 'member-a', '2026-04-08', 11.2, '11.2', 1, '2026-04-08T00:00:00+08:00'),
                    ('iv-b', 'ocr-b', 'record-b', 'proj-blood', 'ind-wbc', 'user-1', 'member-b', '2026-04-08', 15.0, '15.0', 1, '2026-04-08T00:00:00+08:00')",
                [],
            )
            .expect("indicator values should seed");

            update_ocr_item_with_conn(
                conn,
                "ocr-a".to_string(),
                0,
                OcrParsedItem {
                    name: "白细胞".to_string(),
                    value: "8.3".to_string(),
                    unit: "10^9/L".to_string(),
                    reference_range: "3.5-9.5".to_string(),
                    is_abnormal: false,
                },
                &member_scope("user-1", "member-a", "本人"),
            )
            .expect("ocr item should update");

            let member_a_value: String = conn
                .query_row(
                    "SELECT value_text FROM indicator_values WHERE owner_user_id = 'user-1' AND member_id = 'member-a' AND ocr_result_id = 'ocr-a'",
                    [],
                    |row| row.get(0),
                )
                .expect("member A indicator should exist");
            let member_b_value: String = conn
                .query_row(
                    "SELECT value_text FROM indicator_values WHERE owner_user_id = 'user-1' AND member_id = 'member-b' AND ocr_result_id = 'ocr-b'",
                    [],
                    |row| row.get(0),
                )
                .expect("member B indicator should exist");

            assert_eq!(member_a_value, "8.3");
            assert_eq!(member_b_value, "15.0");
        }
        cleanup_test_database(&db, dir);
    }
}
