use serde::{Deserialize, Serialize};
use tauri::Manager;
use crate::db::Database;
use crate::services::http_client;
use crate::commands::ocr::OcrParsedItem;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiAnalysis {
    pub id: String,
    pub record_id: String,
    pub request_prompt: String,
    pub response_content: String,
    pub model_used: String,
    pub status: String,
    pub error_message: String,
    pub created_at: String,
}

/// 发起 AI 分析（流式返回）
#[tauri::command]
pub async fn start_ai_analysis(
    record_id: String,
    app: tauri::AppHandle,
    db: tauri::State<'_, Database>,
) -> Result<String, String> {
    use tauri::Emitter;
    use futures_util::StreamExt;

    // 1. 收集数据：当前 OCR 结果 + 历史数据
    let (config, model, ai_prompt, prompt_data, analysis_id) = {
        let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
        let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

        // 获取 AI 配置
        let config = http_client::load_ai_config(&conn)?;
        let model = http_client::get_default_model(&conn);

        // 获取 AI 分析 Prompt 模板
        let ai_prompt: String = conn
            .query_row(
                "SELECT config_value FROM system_config WHERE config_key = 'ai_analysis_prompt_template'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| {
                "请根据以下检查数据，综合分析患者的健康状况，指出异常指标，提供治疗建议和生活方式改善方案。请以中文回复，使用Markdown格式。".to_string()
            });

        // 获取当前检查记录的 OCR 数据
        let checkup_date: String = conn
            .query_row(
                "SELECT checkup_date FROM checkup_records WHERE id = ?1",
                [&record_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("记录不存在: {}", e))?;

        // 收集当前记录的 OCR 解析结果
        let mut stmt = conn
            .prepare(
                "SELECT o.parsed_items, p.name as project_name, o.checkup_date
                 FROM ocr_results o
                 LEFT JOIN checkup_projects p ON o.project_id = p.id
                 WHERE o.record_id = ?1 AND o.status = 'success'
                 ORDER BY p.name ASC"
            )
            .map_err(|e| format!("查询OCR结果失败: {}", e))?;

        let current_data: Vec<(String, String, String)> = stmt
            .query_map([&record_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1).unwrap_or_default(),
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| format!("查询失败: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("解析数据失败: {}", e))?;

        if current_data.is_empty() {
            return Err("当前检查记录没有成功的 OCR 结果，请先进行 OCR 识别".into());
        }

        // 收集历史检查数据（最近3次有结果的记录）
        // 1. 获取最近 3 次其他检查记录ID
        let mut hist_rec_stmt = conn.prepare(
            "SELECT id, checkup_date FROM checkup_records 
             WHERE id != ?1 
             ORDER BY checkup_date DESC LIMIT 3"
        ).map_err(|e| format!("查询历史记录失败: {}", e))?;
        
        let history_records: Vec<(String, String)> = hist_rec_stmt
            .query_map([&record_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| format!("查询历史记录失败: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_default();

        // 2. 收集这 3 次的历史异常数据
        let mut history_parts = Vec::new();
        for (hist_id, hist_date) in history_records {
             let mut ocr_stmt = conn.prepare(
                 "SELECT o.parsed_items, p.name 
                  FROM ocr_results o
                  LEFT JOIN checkup_projects p ON o.project_id = p.id
                  WHERE o.record_id = ?1 AND o.status = 'success'"
             ).map_err(|e| format!("查询历史OCR失败: {}", e))?;
             
             let ocr_data: Vec<(String, String)> = ocr_stmt.query_map([&hist_id], |row| {
                 Ok((row.get(0)?, row.get(1).unwrap_or_default()))
             }).map_err(|e| format!("查询历史OCR失败: {}", e))?
             .collect::<Result<Vec<_>, _>>()
             .unwrap_or_default();
             
             let mut abnormal_items = Vec::new();
             for (json_str, proj_name) in ocr_data {
                 if let Ok(items) = serde_json::from_str::<Vec<OcrParsedItem>>(&json_str) {
                     for item in items {
                         if item.is_abnormal {
                             abnormal_items.push(format!("- [{}] {}: {} {} (参考: {})", proj_name, item.name, item.value, item.unit, item.reference_range));
                         }
                     }
                 }
             }
             
             if !abnormal_items.is_empty() {
                 history_parts.push(format!("### 日期: {}\n{}", hist_date, abnormal_items.join("\n")));
             }
        }

        // 3. 获取上一次成功的 AI 分析建议
        let last_ai_suggestion: Option<String> = conn.query_row(
            "SELECT response_content FROM ai_analyses a 
             JOIN checkup_records r ON a.record_id = r.id
             WHERE r.id != ?1 AND a.status = 'success'
             ORDER BY r.checkup_date DESC, a.created_at DESC LIMIT 1",
            [&record_id], 
            |row| row.get(0)
        ).ok();

        // 组装完整的 Prompt 数据
        let mut prompt_parts = Vec::new();
        prompt_parts.push(format!("## 1. 本次检查结果（检查日期: {}）\n", checkup_date));

        for (parsed_items, project_name, _date) in &current_data {
            prompt_parts.push(format!("### {}\n", project_name));
            prompt_parts.push(format!("{}\n", parsed_items));
        }

        if !history_parts.is_empty() {
            prompt_parts.push("\n## 2. 近期历史异常记录（仅供参考对比）\n".to_string());
            prompt_parts.push(history_parts.join("\n"));
        }
        
        if let Some(suggestion) = last_ai_suggestion {
            if !suggestion.trim().is_empty() {
                 prompt_parts.push("\n## 3. 上次 AI 分析建议（仅供参考）\n".to_string());
                 prompt_parts.push(format!("{}\n", suggestion));
            }
        }

        let prompt_data = prompt_parts.join("");

        // 预创建分析记录
        let analysis_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Local::now().to_rfc3339();
        let full_prompt = format!("{}\n\n{}", ai_prompt, prompt_data);

        conn.execute(
            "INSERT INTO ai_analyses (id, record_id, request_prompt, response_content, model_used, status, error_message, created_at)
             VALUES (?1, ?2, ?3, '', ?4, 'processing', '', ?5)",
            rusqlite::params![analysis_id, record_id, full_prompt, model, now],
        ).map_err(|e| format!("创建分析记录失败: {}", e))?;

        // 更新检查记录状态
        conn.execute(
            "UPDATE checkup_records SET status = 'ai_processing', updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, record_id],
        ).ok();

        (config, model, ai_prompt, prompt_data, analysis_id)
    };

    let record_id_clone = record_id.clone();
    let analysis_id_clone = analysis_id.clone();

    // 2. 异步发送流式请求
    tokio::spawn(async move {
        let client = match http_client::build_client(&config) {
            Ok(c) => c,
            Err(e) => {
                log::error!("AI 创建客户端失败: {}", e);
                update_ai_error(&app, &analysis_id_clone, &record_id_clone, &e);
                return;
            }
        };

        let full_prompt = format!("{}\n\n{}", ai_prompt, prompt_data);

        let request_body = serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": "你是一位专业的医疗健康分析助手。请根据用户提供的检查报告数据，给出全面、专业的健康分析和建议。"
                },
                {
                    "role": "user",
                    "content": full_prompt
                }
            ],
            "stream": true,
            "max_tokens": 8192,
        });

        let response = match client
            .post(&config.api_url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                let err_msg = format!("AI 请求失败: {}", e);
                update_ai_error(&app, &analysis_id_clone, &record_id_clone, &err_msg);
                return;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let err_msg = format!("AI API 错误 ({}): {}", status, body);
            update_ai_error(&app, &analysis_id_clone, &record_id_clone, &err_msg);
            return;
        }

        // 流式读取 SSE 响应
        let mut full_content = String::new();
        let mut stream = response.bytes_stream();

        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = match chunk_result {
                Ok(c) => c,
                Err(e) => {
                    log::error!("流式读取错误: {}", e);
                    break;
                }
            };

            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);

            // 处理 SSE 数据行
            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() || line == "data: [DONE]" {
                    continue;
                }

                if let Some(json_str) = line.strip_prefix("data: ") {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(json_str) {
                        if let Some(content) = data["choices"][0]["delta"]["content"].as_str() {
                            full_content.push_str(content);
                            // 发送流式 chunk 事件
                            app.emit("ai_stream_chunk", serde_json::json!({
                                "record_id": record_id_clone,
                                "analysis_id": analysis_id_clone,
                                "content": content,
                            })).ok();
                        }
                    }
                }
            }
        }

        // 保存完成的分析结果
        if let Some(db_state) = app.try_state::<Database>() {
            if let Ok(conn_guard) = db_state.conn.lock() {
                if let Some(conn) = conn_guard.as_ref() {
                let now = chrono::Local::now().to_rfc3339();

                let _: Result<usize, _> = conn.execute(
                    "UPDATE ai_analyses SET response_content = ?1, status = 'success', error_message = '' WHERE id = ?2",
                    rusqlite::params![full_content, analysis_id_clone],
                );

                let _: Result<usize, _> = conn.execute(
                    "UPDATE checkup_records SET status = 'ai_done', updated_at = ?1 WHERE id = ?2",
                    rusqlite::params![now, record_id_clone],
                );
                }
            }
        }

        // 发送完成事件
        app.emit("ai_stream_done", serde_json::json!({
            "record_id": record_id_clone,
            "analysis_id": analysis_id_clone,
        })).ok();
    });

    Ok(analysis_id)
}

/// 获取 AI 分析结果
#[tauri::command]
pub fn get_ai_analysis(record_id: String, db: tauri::State<Database>) -> Result<Vec<AiAnalysis>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, record_id, request_prompt, response_content, model_used, status, error_message, created_at
             FROM ai_analyses WHERE record_id = ?1
             ORDER BY created_at DESC"
        )
        .map_err(|e| format!("查询AI分析结果失败: {}", e))?;

    let results = stmt
        .query_map([&record_id], |row| {
            Ok(AiAnalysis {
                id: row.get(0)?,
                record_id: row.get(1)?,
                request_prompt: row.get(2)?,
                response_content: row.get(3)?,
                model_used: row.get(4)?,
                status: row.get(5)?,
                error_message: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("查询失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析数据失败: {}", e))?;

    Ok(results)
}

/// 更新 AI 分析错误状态
fn update_ai_error(app: &tauri::AppHandle, analysis_id: &str, record_id: &str, error: &str) {
    use tauri::Emitter;

    if let Some(db_state) = app.try_state::<Database>() {
        if let Ok(conn_guard) = db_state.conn.lock() {
            if let Some(conn) = conn_guard.as_ref() {
            let now = chrono::Local::now().to_rfc3339();

            let _: Result<usize, _> = conn.execute(
                "UPDATE ai_analyses SET status = 'failed', error_message = ?1 WHERE id = ?2",
                rusqlite::params![error, analysis_id],
            );

            let _: Result<usize, _> = conn.execute(
                "UPDATE checkup_records SET status = 'ocr_done', updated_at = ?1 WHERE id = ?2",
                rusqlite::params![now, record_id],
            );
            }
        }
    }

    app.emit("ai_stream_error", serde_json::json!({
        "record_id": record_id,
        "analysis_id": analysis_id,
        "error": error,
    })).ok();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiAnalysisHistoryItem {
    pub id: String,
    pub record_id: String,
    pub checkup_date: String,
    pub response_content: String,
    pub created_at: String,
}

/// 获取历史 AI 分析记录（分页）
#[tauri::command]
pub fn get_ai_analyses_history(
    page: i64,
    size: i64,
    db: tauri::State<Database>,
) -> Result<Vec<AiAnalysisHistoryItem>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    let offset = (page - 1) * size;
    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.record_id, r.checkup_date, a.response_content, a.created_at
             FROM ai_analyses a
             JOIN checkup_records r ON a.record_id = r.id
             WHERE a.status = 'success'
             ORDER BY r.checkup_date DESC, a.created_at DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    let results = stmt
        .query_map(rusqlite::params![size, offset], |row| {
            Ok(AiAnalysisHistoryItem {
                id: row.get(0)?,
                record_id: row.get(1)?,
                checkup_date: row.get(2)?,
                response_content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| format!("查询失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析数据失败: {}", e))?;

    Ok(results)
}

/// 更新 AI 分析内容（编辑保存）
#[tauri::command]
pub fn update_ai_analysis_content(
    id: String,
    content: String,
    db: tauri::State<Database>,
) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    conn.execute(
        "UPDATE ai_analyses SET response_content = ?1 WHERE id = ?2",
        rusqlite::params![content, id],
    )
    .map_err(|e| format!("更新失败: {}", e))?;

    Ok(true)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

/// 获取聊天历史
#[tauri::command]
pub fn get_chat_history(
    limit: i64,
    offset: i64,
    db: tauri::State<Database>,
) -> Result<Vec<ChatMessage>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    // Return DESC order (newest first) for easier pagination
    let mut stmt = conn
        .prepare(
            "SELECT id, role, content, created_at FROM chat_logs 
             ORDER BY created_at DESC, role ASC 
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    let results = stmt
        .query_map(rusqlite::params![limit, offset], |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                role: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| format!("查询失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析数据失败: {}", e))?;

    Ok(results)
}

/// 清空聊天历史
#[tauri::command]
pub fn clear_chat_history(db: tauri::State<Database>) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    conn.execute("DELETE FROM chat_logs", [])
        .map_err(|e| format!("删除失败: {}", e))?;

    Ok(true)
}

/// 与 AI 对话（流式）
#[tauri::command]
pub async fn chat_with_ai(
    message: String,
    app: tauri::AppHandle,
    db: tauri::State<'_, Database>,
) -> Result<String, String> {
    use tauri::Emitter;
    use futures_util::StreamExt;

    // 1. 获取配置和上下文
    let (config, model, chat_history) = {
        let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
        let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

        let config = http_client::load_ai_config(&conn)?;
        let model = http_client::get_default_model(&conn);

        // 获取最近 10 条历史记录作为上下文
        let mut stmt = conn.prepare("SELECT role, content FROM chat_logs ORDER BY created_at DESC, role ASC LIMIT 10")
            .map_err(|e| e.to_string())?;
        
        let mut history: Vec<(String, String)> = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).map_err(|e| e.to_string())?
          .collect::<Result<Vec<_>, _>>()
          .map_err(|e| e.to_string())?;
          
        history.reverse(); // 恢复时间顺序
        (config, model, history)
    };

    let user_msg_id = uuid::Uuid::new_v4().to_string();
    let ai_msg_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().to_rfc3339();

    // 2. 保存用户消息
    {
         let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
         let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
         
         conn.execute(
             "INSERT INTO chat_logs (id, role, content, created_at) VALUES (?1, 'user', ?2, ?3)",
             rusqlite::params![user_msg_id, message, now],
         ).map_err(|e| e.to_string())?;
         
         // 预创建 AI 回复记录 (content empty)
         conn.execute(
             "INSERT INTO chat_logs (id, role, content, created_at) VALUES (?1, 'assistant', '', ?2)",
             rusqlite::params![ai_msg_id, now],
         ).map_err(|e| e.to_string())?;
    }

    // 3. 构造请求
    let mut messages = Vec::new();
    messages.push(serde_json::json!({
        "role": "system",
        "content": "你是一位专业的医疗健康助手。请简明扼要地回答用户的问题。"
    }));

    for (role, content) in chat_history {
        messages.push(serde_json::json!({ "role": role, "content": content }));
    }
    messages.push(serde_json::json!({ "role": "user", "content": message }));

    let request_body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "max_tokens": 4096,
    });

    let ai_msg_id_clone = ai_msg_id.clone();
    
    // 4. 发送请求
    tokio::spawn(async move {
        let client = match http_client::build_client(&config) {
            Ok(c) => c,
            Err(e) => {
                 app.emit("chat_stream_error", serde_json::json!({"id": ai_msg_id_clone, "error": format!("Client Error: {}", e)})).ok();
                 return;
            }
        };

        let response = match client
            .post(&config.api_url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await 
        {
            Ok(r) => r,
             Err(e) => {
                 app.emit("chat_stream_error", serde_json::json!({"id": ai_msg_id_clone, "error": format!("Request Error: {}", e)})).ok();
                 return;
            }
        };

        if !response.status().is_success() {
             app.emit("chat_stream_error", serde_json::json!({"id": ai_msg_id_clone, "error": format!("API Error: {}", response.status())})).ok();
             return;
        }

        let mut stream = response.bytes_stream();
        // Removed `full_content` initialization here, using buffer for lines
        let mut full_response_content = String::new();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = match chunk_result {
                Ok(c) => c,
                Err(_) => break,
            };
            
            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);

            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() || line == "data: [DONE]" { continue; }

                if let Some(json_str) = line.strip_prefix("data: ") {
                     if let Ok(data) = serde_json::from_str::<serde_json::Value>(json_str) {
                        if let Some(content) = data["choices"][0]["delta"]["content"].as_str() {
                            full_response_content.push_str(content);
                            app.emit("chat_stream_chunk", serde_json::json!({
                                "id": ai_msg_id_clone,
                                "content": content
                            })).ok();
                        }
                     }
                }
            }
        }

        // Update DB with full content
        if let Some(db_state) = app.try_state::<Database>() {
            if let Ok(conn_guard) = db_state.conn.lock() {
                if let Some(conn) = conn_guard.as_ref() {
                    let _ = conn.execute(
                        "UPDATE chat_logs SET content = ?1 WHERE id = ?2",
                        rusqlite::params![full_response_content, ai_msg_id_clone],
                    );
                }
            }
        }
        
        app.emit("chat_stream_done", serde_json::json!({"id": ai_msg_id_clone})).ok();
    });

    Ok(ai_msg_id)
}
