use super::scope::{
    resolve_chat_scope, resolve_member_scope, touch_conversation, ChatScopeInput,
    MemberScopeInput, ResolvedChatScope, ResolvedMemberScope,
};
use crate::commands::ocr::OcrParsedItem;
use crate::db::Database;
use rusqlite::Connection;
use crate::services::http_client;
use serde::{Deserialize, Serialize};
use tauri::Manager;

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

fn extract_text_segments_from_ai_chunk(chunk: &serde_json::Value) -> Vec<String> {
    let mut segments = Vec::new();

    let push_content_value = |segments: &mut Vec<String>, content: &serde_json::Value| {
        if let Some(s) = content.as_str() {
            if !s.is_empty() {
                segments.push(s.to_string());
            }
            return;
        }

        if let Some(arr) = content.as_array() {
            for item in arr {
                if let Some(s) = item.as_str() {
                    if !s.is_empty() {
                        segments.push(s.to_string());
                    }
                    continue;
                }

                if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                    if !t.is_empty() {
                        segments.push(t.to_string());
                    }
                } else if let Some(t) = item.get("content").and_then(|v| v.as_str()) {
                    if !t.is_empty() {
                        segments.push(t.to_string());
                    }
                }
            }
        }
    };

    if let Some(content) = chunk
        .get("choices")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("delta"))
        .and_then(|v| v.get("content"))
    {
        push_content_value(&mut segments, content);
    }

    if let Some(content) = chunk
        .get("choices")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("message"))
        .and_then(|v| v.get("content"))
    {
        push_content_value(&mut segments, content);
    }

    if let Some(text) = chunk.get("output_text").and_then(|v| v.as_str()) {
        if !text.is_empty() {
            segments.push(text.to_string());
        }
    }

    segments
}

fn extract_text_segments_from_stream_line(line: &str) -> (Vec<String>, bool) {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return (Vec::new(), false);
    }

    let payload = if let Some(v) = trimmed.strip_prefix("data:") {
        v.trim()
    } else {
        trimmed
    };

    if payload == "[DONE]" {
        return (Vec::new(), true);
    }

    if payload.is_empty() {
        return (Vec::new(), false);
    }

    match serde_json::from_str::<serde_json::Value>(payload) {
        Ok(chunk) => (extract_text_segments_from_ai_chunk(&chunk), false),
        Err(_) => (Vec::new(), false),
    }
}

/// 发起 AI 分析（流式返回）
#[tauri::command]
pub async fn start_ai_analysis(
    record_id: String,
    scope: Option<MemberScopeInput>,
    app: tauri::AppHandle,
    db: tauri::State<'_, Database>,
) -> Result<String, String> {
    use futures_util::StreamExt;
    use tauri::Emitter;

    // 1. 收集数据：当前 OCR 结果 + 历史数据
    let (config, model, ai_prompt, prompt_data, analysis_id) = {
        let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
        let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
        let scope = resolve_member_scope(conn, scope)?;

        // 获取 AI 配置
        let config = http_client::load_ai_config(&conn)?;
        let model = http_client::get_default_model(&conn);

        // 获取 AI 分析 Prompt 模板
        let ai_analysis_prompt: String = conn
            .query_row(
                "SELECT config_value FROM system_config WHERE config_key = 'ai_analysis_prompt_template'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| {
                "你是一位专业的医疗健康分析师。请根据以下检查数据，综合分析患者的健康状况，指出异常指标，提供治疗建议和生活方式改善方案。请以中文回复，使用Markdown格式。".to_string()
            });

        // 获取用户自定义 Prompt（患者情况说明）
        let user_custom_prompt: String = conn
            .query_row(
                "SELECT config_value FROM system_config WHERE config_key = 'user_custom_prompt_template'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_default();

        // 拼接系统级 Prompt：用户自定义在前，AI 分析指令在后
        let ai_prompt = if user_custom_prompt.trim().is_empty() {
            ai_analysis_prompt
        } else {
            format!("{}\n\n{}", user_custom_prompt.trim(), ai_analysis_prompt.trim())
        };

        // 获取当前检查记录的 OCR 数据
        let checkup_date: String = conn
            .query_row(
                "SELECT checkup_date
                 FROM checkup_records
                 WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3",
                rusqlite::params![&record_id, &scope.owner_user_id, &scope.member_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("记录不存在: {}", e))?;

        // 收集当前记录的 OCR 解析结果
        let mut stmt = conn
            .prepare(
                "SELECT o.parsed_items, p.name as project_name, o.checkup_date
                 FROM ocr_results o
                 LEFT JOIN checkup_projects p ON o.project_id = p.id
                 WHERE o.record_id = ?1
                   AND o.owner_user_id = ?2
                   AND o.member_id = ?3
                   AND o.status = 'success'
                 ORDER BY p.name ASC",
            )
            .map_err(|e| format!("查询OCR结果失败: {}", e))?;

        let current_data: Vec<(String, String, String)> = stmt
            .query_map(
                rusqlite::params![&record_id, &scope.owner_user_id, &scope.member_id],
                |row| {
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
        let mut hist_rec_stmt = conn
            .prepare(
                "SELECT id, checkup_date FROM checkup_records 
             WHERE id != ?1 AND owner_user_id = ?2 AND member_id = ?3
             ORDER BY checkup_date DESC LIMIT 3",
            )
            .map_err(|e| format!("查询历史记录失败: {}", e))?;

        let history_records: Vec<(String, String)> = hist_rec_stmt
            .query_map(
                rusqlite::params![&record_id, &scope.owner_user_id, &scope.member_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| format!("查询历史记录失败: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_default();

        // 2. 收集这 3 次的历史异常数据
        let mut history_parts = Vec::new();
        for (hist_id, hist_date) in history_records {
            let mut ocr_stmt = conn
                .prepare(
                    "SELECT o.parsed_items, p.name 
                  FROM ocr_results o
                  LEFT JOIN checkup_projects p ON o.project_id = p.id
                  WHERE o.record_id = ?1
                    AND o.owner_user_id = ?2
                    AND o.member_id = ?3
                    AND o.status = 'success'",
                )
                .map_err(|e| format!("查询历史OCR失败: {}", e))?;

            let ocr_data: Vec<(String, String)> = ocr_stmt
                .query_map(
                    rusqlite::params![&hist_id, &scope.owner_user_id, &scope.member_id],
                    |row| {
                    Ok((row.get(0)?, row.get(1).unwrap_or_default()))
                })
                .map_err(|e| format!("查询历史OCR失败: {}", e))?
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_default();

            let mut abnormal_items = Vec::new();
            for (json_str, proj_name) in ocr_data {
                if let Ok(items) = serde_json::from_str::<Vec<OcrParsedItem>>(&json_str) {
                    for item in items {
                        if item.is_abnormal {
                            abnormal_items.push(format!(
                                "- [{}] {}: {} {} (参考: {})",
                                proj_name, item.name, item.value, item.unit, item.reference_range
                            ));
                        }
                    }
                }
            }

            if !abnormal_items.is_empty() {
                history_parts.push(format!(
                    "### 日期: {}\n{}",
                    hist_date,
                    abnormal_items.join("\n")
                ));
            }
        }

        // 3. 获取上一次成功的 AI 分析建议
        let last_ai_suggestion: Option<String> = conn
            .query_row(
            "SELECT response_content FROM ai_analyses a 
             JOIN checkup_records r ON a.record_id = r.id
             WHERE r.id != ?1
               AND a.owner_user_id = ?2
               AND a.member_id = ?3
               AND a.status = 'success'
             ORDER BY r.checkup_date DESC, a.created_at DESC LIMIT 1",
                rusqlite::params![&record_id, &scope.owner_user_id, &scope.member_id],
                |row| row.get(0),
            )
            .ok();

        // 组装完整的 Prompt 数据
        let mut prompt_parts = Vec::new();
        prompt_parts.push(format!(
            "## 1. 本次检查结果（检查日期: {}）\n",
            checkup_date
        ));

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
            "INSERT INTO ai_analyses (id, owner_user_id, member_id, record_id, request_prompt, response_content, model_used, status, error_message, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, '', ?6, 'processing', '', ?7)",
            rusqlite::params![
                analysis_id,
                &scope.owner_user_id,
                &scope.member_id,
                record_id,
                full_prompt,
                model,
                now
            ],
        ).map_err(|e| format!("创建分析记录失败: {}", e))?;

        // 更新检查记录状态
        conn.execute(
            "UPDATE checkup_records
             SET status = 'ai_processing', updated_at = ?1
             WHERE id = ?2 AND owner_user_id = ?3 AND member_id = ?4",
            rusqlite::params![now, record_id, &scope.owner_user_id, &scope.member_id],
        )
        .ok();

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
        let mut done = false;

        while !done {
            let chunk_result = match stream.next().await {
                Some(v) => v,
                None => break,
            };
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

                let (segments, is_done) = extract_text_segments_from_stream_line(&line);
                for content in segments {
                    full_content.push_str(&content);
                    app.emit(
                        "ai_stream_chunk",
                        serde_json::json!({
                            "record_id": record_id_clone,
                            "analysis_id": analysis_id_clone,
                            "content": content,
                        }),
                    )
                    .ok();
                }
                if is_done {
                    done = true;
                    break;
                }
            }
        }

        if !done && !buffer.trim().is_empty() {
            let (segments, _) = extract_text_segments_from_stream_line(buffer.trim());
            for content in segments {
                full_content.push_str(&content);
                app.emit(
                    "ai_stream_chunk",
                    serde_json::json!({
                        "record_id": record_id_clone,
                        "analysis_id": analysis_id_clone,
                        "content": content,
                    }),
                )
                .ok();
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
        app.emit(
            "ai_stream_done",
            serde_json::json!({
                "record_id": record_id_clone,
                "analysis_id": analysis_id_clone,
            }),
        )
        .ok();
    });

    Ok(analysis_id)
}

/// 获取 AI 分析结果
fn get_ai_analysis_with_conn(
    conn: &Connection,
    record_id: String,
    scope: &ResolvedMemberScope,
) -> Result<Vec<AiAnalysis>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, record_id, request_prompt, response_content, model_used, status, error_message, created_at
             FROM ai_analyses
             WHERE record_id = ?1 AND owner_user_id = ?2 AND member_id = ?3
             ORDER BY created_at DESC"
        )
        .map_err(|e| format!("查询AI分析结果失败: {}", e))?;

    let results = stmt
        .query_map(
            rusqlite::params![&record_id, &scope.owner_user_id, &scope.member_id],
            |row| {
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

#[tauri::command]
pub fn get_ai_analysis(
    record_id: String,
    scope: Option<MemberScopeInput>,
    db: tauri::State<Database>,
) -> Result<Vec<AiAnalysis>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    get_ai_analysis_with_conn(conn, record_id, &scope)
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

    app.emit(
        "ai_stream_error",
        serde_json::json!({
            "record_id": record_id,
            "analysis_id": analysis_id,
            "error": error,
        }),
    )
    .ok();
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
fn get_ai_analyses_history_with_conn(
    conn: &Connection,
    page: i64,
    size: i64,
    scope: &ResolvedMemberScope,
) -> Result<Vec<AiAnalysisHistoryItem>, String> {
    let offset = (page - 1) * size;
    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.record_id, r.checkup_date, a.response_content, a.created_at
             FROM ai_analyses a
             JOIN checkup_records r ON a.record_id = r.id
             WHERE a.status = 'success'
               AND a.owner_user_id = ?1
               AND a.member_id = ?2
             ORDER BY r.checkup_date DESC, a.created_at DESC
             LIMIT ?3 OFFSET ?4",
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    let results = stmt
        .query_map(
            rusqlite::params![&scope.owner_user_id, &scope.member_id, size, offset],
            |row| {
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

#[tauri::command]
pub fn get_ai_analyses_history(
    page: i64,
    size: i64,
    scope: Option<MemberScopeInput>,
    db: tauri::State<Database>,
) -> Result<Vec<AiAnalysisHistoryItem>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    get_ai_analyses_history_with_conn(conn, page, size, &scope)
}

/// 更新 AI 分析内容（编辑保存）
fn update_ai_analysis_content_with_conn(
    conn: &Connection,
    id: String,
    content: String,
    scope: &ResolvedMemberScope,
) -> Result<bool, String> {
    conn.execute(
        "UPDATE ai_analyses
         SET response_content = ?1
         WHERE id = ?2 AND owner_user_id = ?3 AND member_id = ?4",
        rusqlite::params![content, id, &scope.owner_user_id, &scope.member_id],
    )
    .map_err(|e| format!("更新失败: {}", e))?;

    Ok(true)
}

#[tauri::command]
pub fn update_ai_analysis_content(
    id: String,
    content: String,
    scope: Option<MemberScopeInput>,
    db: tauri::State<Database>,
) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let scope = resolve_member_scope(conn, scope)?;
    update_ai_analysis_content_with_conn(conn, id, content, &scope)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

/// 获取聊天历史
fn get_chat_history_with_conn(
    conn: &Connection,
    limit: i64,
    offset: i64,
    chat_scope: &ResolvedChatScope,
) -> Result<Vec<ChatMessage>, String> {
    // Return DESC order (newest first) for easier pagination
    let mut stmt = conn
        .prepare(
            "SELECT id, role, content, created_at
             FROM chat_logs
             WHERE owner_user_id = ?1 AND member_id = ?2 AND conversation_id = ?3
             ORDER BY created_at DESC, role ASC 
             LIMIT ?4 OFFSET ?5",
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    let results = stmt
        .query_map(
            rusqlite::params![
                &chat_scope.owner_user_id,
                &chat_scope.member_id,
                &chat_scope.conversation_id,
                limit,
                offset
            ],
            |row| {
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

#[tauri::command]
pub fn get_chat_history(
    limit: i64,
    offset: i64,
    scope: Option<ChatScopeInput>,
    db: tauri::State<Database>,
) -> Result<Vec<ChatMessage>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let chat_scope = resolve_chat_scope(conn, scope)?;
    get_chat_history_with_conn(conn, limit, offset, &chat_scope)
}

/// 清空聊天历史
fn clear_chat_history_with_conn(
    conn: &Connection,
    chat_scope: &ResolvedChatScope,
) -> Result<bool, String> {
    conn.execute(
        "DELETE FROM chat_logs
         WHERE owner_user_id = ?1 AND member_id = ?2 AND conversation_id = ?3",
        rusqlite::params![
            &chat_scope.owner_user_id,
            &chat_scope.member_id,
            &chat_scope.conversation_id
        ],
    )
        .map_err(|e| format!("删除失败: {}", e))?;

    touch_conversation(conn, &chat_scope.conversation_id)?;

    Ok(true)
}

#[tauri::command]
pub fn clear_chat_history(
    scope: Option<ChatScopeInput>,
    db: tauri::State<Database>,
) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let chat_scope = resolve_chat_scope(conn, scope)?;
    clear_chat_history_with_conn(conn, &chat_scope)
}

/// 与 AI 对话（流式）
#[tauri::command]
pub async fn chat_with_ai(
    message: String,
    scope: Option<ChatScopeInput>,
    app: tauri::AppHandle,
    db: tauri::State<'_, Database>,
) -> Result<String, String> {
    use futures_util::StreamExt;
    use tauri::Emitter;

    // 1. 获取配置和上下文
    let (config, model, chat_history, system_prompt, chat_scope) = {
        let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
        let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
        let chat_scope = resolve_chat_scope(conn, scope)?;

        let config = http_client::load_ai_config(&conn)?;
        let model = http_client::get_default_model(&conn);

        // 获取用户自定义 Prompt（患者情况说明）
        let user_custom_prompt: String = conn
            .query_row(
                "SELECT config_value FROM system_config WHERE config_key = 'user_custom_prompt_template'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_default();

        // 获取 AI 分析 Prompt 模板
        let ai_analysis_prompt: String = conn
            .query_row(
                "SELECT config_value FROM system_config WHERE config_key = 'ai_analysis_prompt_template'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| {
                "请根据以下检查数据，综合分析患者的健康状况，指出异常指标，提供治疗建议和生活方式改善方案。请以中文回复，使用Markdown格式。".to_string()
            });

        // 拼接系统 Prompt：用户自定义在前，AI 分析在后
        let system_prompt = if user_custom_prompt.trim().is_empty() {
            ai_analysis_prompt
        } else {
            format!("{}\n\n{}", user_custom_prompt.trim(), ai_analysis_prompt.trim())
        };

        // 获取最近 10 条历史记录作为上下文
        let mut stmt = conn
            .prepare(
                "SELECT role, content
                 FROM chat_logs
                 WHERE owner_user_id = ?1 AND member_id = ?2 AND conversation_id = ?3
                 ORDER BY created_at DESC, role ASC
                 LIMIT 10",
            )
            .map_err(|e| e.to_string())?;

        let mut history: Vec<(String, String)> = stmt
            .query_map(
                rusqlite::params![
                    &chat_scope.owner_user_id,
                    &chat_scope.member_id,
                    &chat_scope.conversation_id
                ],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        history.reverse(); // 恢复时间顺序
        (config, model, history, system_prompt, chat_scope)
    };

    let user_msg_id = uuid::Uuid::new_v4().to_string();
    let ai_msg_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().to_rfc3339();

    // 2. 保存用户消息
    {
        let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
        let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

        conn.execute(
            "INSERT INTO chat_logs (id, owner_user_id, member_id, conversation_id, role, content, created_at)
             VALUES (?1, ?2, ?3, ?4, 'user', ?5, ?6)",
            rusqlite::params![
                user_msg_id,
                &chat_scope.owner_user_id,
                &chat_scope.member_id,
                &chat_scope.conversation_id,
                message,
                now
            ],
        )
        .map_err(|e| e.to_string())?;

        // 预创建 AI 回复记录 (content empty)
        conn.execute(
             "INSERT INTO chat_logs (id, owner_user_id, member_id, conversation_id, role, content, created_at)
              VALUES (?1, ?2, ?3, ?4, 'assistant', '', ?5)",
             rusqlite::params![
                ai_msg_id,
                &chat_scope.owner_user_id,
                &chat_scope.member_id,
                &chat_scope.conversation_id,
                now
             ],
         ).map_err(|e| e.to_string())?;
        touch_conversation(conn, &chat_scope.conversation_id)?;
    }

    // 3. 构造请求
    let mut messages = Vec::new();
    messages.push(serde_json::json!({
        "role": "system",
        "content": system_prompt
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
    let conversation_id = chat_scope.conversation_id.clone();

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
        let mut done = false;

        while !done {
            let chunk_result = match stream.next().await {
                Some(v) => v,
                None => break,
            };
            let chunk = match chunk_result {
                Ok(c) => c,
                Err(_) => break,
            };

            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);

            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                let (segments, is_done) = extract_text_segments_from_stream_line(&line);
                for content in segments {
                    full_response_content.push_str(&content);
                    app.emit(
                        "chat_stream_chunk",
                        serde_json::json!({
                            "id": ai_msg_id_clone,
                            "content": content
                        }),
                    )
                    .ok();
                }
                if is_done {
                    done = true;
                    break;
                }
            }
        }

        if !done && !buffer.trim().is_empty() {
            let (segments, _) = extract_text_segments_from_stream_line(buffer.trim());
            for content in segments {
                full_response_content.push_str(&content);
                app.emit(
                    "chat_stream_chunk",
                    serde_json::json!({
                        "id": ai_msg_id_clone,
                        "content": content
                    }),
                )
                .ok();
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
                    let _ = touch_conversation(conn, &conversation_id);
                }
            }
        }

        app.emit(
            "chat_stream_done",
            serde_json::json!({"id": ai_msg_id_clone}),
        )
        .ok();
    });

    Ok(ai_msg_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_database(name: &str) -> (Database, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "health-monitor-ai-tests-{}-{}",
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

    fn member_scope(owner_user_id: &str, member_id: &str, member_name: &str) -> ResolvedMemberScope {
        ResolvedMemberScope {
            owner_user_id: owner_user_id.to_string(),
            member_id: member_id.to_string(),
            member_name: member_name.to_string(),
        }
    }

    fn chat_scope(
        owner_user_id: &str,
        member_id: &str,
        member_name: &str,
        conversation_id: &str,
    ) -> ResolvedChatScope {
        ResolvedChatScope {
            owner_user_id: owner_user_id.to_string(),
            member_id: member_id.to_string(),
            member_name: member_name.to_string(),
            conversation_id: conversation_id.to_string(),
        }
    }

    fn seed_analysis_record(
        conn: &Connection,
        owner_user_id: &str,
        member_id: &str,
        record_id: &str,
        analysis_id: &str,
        checkup_date: &str,
        content: &str,
    ) {
        conn.execute(
            "INSERT INTO checkup_records (
                id, owner_user_id, member_id, member_name_snapshot, checkup_date, status, notes, created_at, updated_at
             ) VALUES (?1, ?2, ?3, '成员', ?4, 'ai_done', '', '2026-04-08T00:00:00+08:00', '2026-04-08T00:00:00+08:00')",
            rusqlite::params![record_id, owner_user_id, member_id, checkup_date],
        )
        .expect("record should seed");
        conn.execute(
            "INSERT INTO ai_analyses (
                id, owner_user_id, member_id, record_id, request_prompt, response_content, model_used, status, error_message, created_at
             ) VALUES (?1, ?2, ?3, ?4, '', ?5, 'gpt-test', 'success', '', '2026-04-08T00:00:00+08:00')",
            rusqlite::params![analysis_id, owner_user_id, member_id, record_id, content],
        )
        .expect("analysis should seed");
    }

    fn seed_conversation(
        conn: &Connection,
        conversation_id: &str,
        owner_user_id: &str,
        member_id: &str,
    ) {
        conn.execute(
            "INSERT INTO chat_conversations (id, owner_user_id, member_id, title, created_at, updated_at)
             VALUES (?1, ?2, ?3, '默认会话', '2026-04-08T00:00:00+08:00', '2026-04-08T00:00:00+08:00')",
            rusqlite::params![conversation_id, owner_user_id, member_id],
        )
        .expect("conversation should seed");
    }

    fn seed_chat_message(
        conn: &Connection,
        id: &str,
        owner_user_id: &str,
        member_id: &str,
        conversation_id: &str,
        role: &str,
        content: &str,
        created_at: &str,
    ) {
        conn.execute(
            "INSERT INTO chat_logs (id, owner_user_id, member_id, conversation_id, role, content, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![id, owner_user_id, member_id, conversation_id, role, content, created_at],
        )
        .expect("chat log should seed");
    }

    #[test]
    fn get_ai_analyses_history_only_returns_current_member_rows() {
        let (db, dir) = create_test_database("analysis-history");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");

            seed_analysis_record(conn, "user-1", "member-a", "record-a", "analysis-a", "2026-04-07", "成员A分析");
            seed_analysis_record(conn, "user-1", "member-b", "record-b", "analysis-b", "2026-04-08", "成员B分析");

            let history = get_ai_analyses_history_with_conn(
                conn,
                1,
                20,
                &member_scope("user-1", "member-a", "本人"),
            )
            .expect("history should load");

            assert_eq!(history.len(), 1);
            assert_eq!(history[0].id, "analysis-a");
            assert_eq!(history[0].record_id, "record-a");
            assert_eq!(history[0].response_content, "成员A分析");
        }
        cleanup_test_database(&db, dir);
    }

    #[test]
    fn get_chat_history_only_returns_requested_member_conversation() {
        let (db, dir) = create_test_database("chat-history");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");

            seed_conversation(conn, "conv-a", "user-1", "member-a");
            seed_conversation(conn, "conv-b", "user-1", "member-b");
            seed_chat_message(conn, "msg-a1", "user-1", "member-a", "conv-a", "user", "成员A提问", "2026-04-08T10:00:00+08:00");
            seed_chat_message(conn, "msg-a2", "user-1", "member-a", "conv-a", "assistant", "成员A回答", "2026-04-08T10:01:00+08:00");
            seed_chat_message(conn, "msg-b1", "user-1", "member-b", "conv-b", "user", "成员B提问", "2026-04-08T10:02:00+08:00");

            let history = get_chat_history_with_conn(
                conn,
                20,
                0,
                &chat_scope("user-1", "member-a", "本人", "conv-a"),
            )
            .expect("chat history should load");

            assert_eq!(history.len(), 2);
            assert!(history.iter().all(|item| item.content.contains('A')));
        }
        cleanup_test_database(&db, dir);
    }

    #[test]
    fn clear_chat_history_only_deletes_current_member_conversation_rows() {
        let (db, dir) = create_test_database("clear-chat");
        {
            let conn_guard = db.conn.lock().expect("lock should succeed");
            let conn = conn_guard.as_ref().expect("conn should exist");

            seed_conversation(conn, "conv-a", "user-1", "member-a");
            seed_conversation(conn, "conv-b", "user-1", "member-b");
            seed_chat_message(conn, "msg-a1", "user-1", "member-a", "conv-a", "user", "成员A提问", "2026-04-08T10:00:00+08:00");
            seed_chat_message(conn, "msg-a2", "user-1", "member-a", "conv-a", "assistant", "成员A回答", "2026-04-08T10:01:00+08:00");
            seed_chat_message(conn, "msg-b1", "user-1", "member-b", "conv-b", "user", "成员B提问", "2026-04-08T10:02:00+08:00");

            clear_chat_history_with_conn(
                conn,
                &chat_scope("user-1", "member-a", "本人", "conv-a"),
            )
            .expect("chat history should clear");

            let remaining_member_a_logs: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM chat_logs WHERE owner_user_id = 'user-1' AND member_id = 'member-a' AND conversation_id = 'conv-a'",
                    [],
                    |row| row.get(0),
                )
                .expect("count should work");
            let remaining_member_b_logs: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM chat_logs WHERE owner_user_id = 'user-1' AND member_id = 'member-b' AND conversation_id = 'conv-b'",
                    [],
                    |row| row.get(0),
                )
                .expect("count should work");

            assert_eq!(remaining_member_a_logs, 0);
            assert_eq!(remaining_member_b_logs, 1);
        }
        cleanup_test_database(&db, dir);
    }
}
