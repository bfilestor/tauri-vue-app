use crate::db::Database;
use crate::services::http_client;
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

/// 发起 OCR 识别（异步执行，通过 Event 通知前端）
#[tauri::command]
pub async fn start_ocr(
    record_id: String,
    app: tauri::AppHandle,
    db: tauri::State<'_, Database>,
    app_dir: tauri::State<'_, super::AppDir>,
) -> Result<String, String> {
    use tauri::Emitter;

    // 1. 查询记录和关联的文件
    let (files, checkup_date, config, model, ocr_prompt, indicators_map) = {
        let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
        let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

        // 获取检查日期
        let checkup_date: String = conn
            .query_row(
                "SELECT checkup_date FROM checkup_records WHERE id = ?1",
                [&record_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("记录不存在: {}", e))?;

        // 获取关联文件
        let mut stmt = conn
            .prepare(
                "SELECT f.id, f.record_id, f.project_id, f.original_filename, f.stored_path, f.mime_type, p.name
                 FROM checkup_files f
                 LEFT JOIN checkup_projects p ON f.project_id = p.id
                 WHERE f.record_id = ?1
                 ORDER BY p.name ASC, f.uploaded_at ASC"
            )
            .map_err(|e| format!("查询文件失败: {}", e))?;

        let files: Vec<(String, String, String, String, String, String, String)> = stmt
            .query_map([&record_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6).unwrap_or_default(),
                ))
            })
            .map_err(|e| format!("查询文件失败: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("解析文件数据失败: {}", e))?;

        if files.is_empty() {
            return Err("该检查记录下没有文件，请先上传检查报告图片".into());
        }

        // 获取 AI 配置
        let config = http_client::load_ai_config(&conn)?;
        let model = http_client::get_default_model(&conn);

        // 获取 OCR Prompt 模板
        let ocr_prompt: String = conn
            .query_row(
                "SELECT config_value FROM system_config WHERE config_key = 'ocr_prompt_template'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "请识别图片中的医疗检查报告，提取所有检查指标。请严格按照以下JSON格式返回数组：[{\"name\":\"指标名称\",\"value\":\"数值\",\"unit\":\"单位\",\"reference_range\":\"参考范围\",\"status\":\"正常/异常\"}]。注意：reference_range字段请统一使用\"reference_range\"作为键名；status字段请依据数值和参考范围判断，仅返回\"正常\"或\"异常\"；如果图片中没有明确状态标记，请根据数值自行判断。只返回JSON数组，不要返回其他内容。".to_string());

        // 加载所有项目的指标映射（用于匹配 indicator_values）
        let mut ind_stmt = conn
            .prepare("SELECT id, project_id, name FROM indicators")
            .map_err(|e| format!("查询指标失败: {}", e))?;
        let indicators: Vec<(String, String, String)> = ind_stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| format!("查询指标失败: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("解析指标数据失败: {}", e))?;

        (files, checkup_date, config, model, ocr_prompt, indicators)
    };

    // 更新状态为 ocr_processing
    {
        let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
        let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
        let now = chrono::Local::now().to_rfc3339();
        conn.execute(
            "UPDATE checkup_records SET status = 'ocr_processing', updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, record_id],
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
                "max_tokens": 4096,
            });

            // 发送请求
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
                    let err_msg = format!("{}: 请求失败 - {}", filename, e);
                    error_messages.push(err_msg.clone());
                    save_ocr_error(
                        &app,
                        &record_id_clone,
                        file_id,
                        &project_id,
                        &checkup_date,
                        &err_msg,
                    );
                    continue;
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                let err_msg = format!("{}: API错误({}) - {}", filename, status, body);
                error_messages.push(err_msg.clone());
                save_ocr_error(
                    &app,
                    &record_id_clone,
                    file_id,
                    &project_id,
                    &checkup_date,
                    &err_msg,
                );
                continue;
            }

            // 解析响应
            let resp_json: serde_json::Value = match response.json().await {
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
                        &err_msg,
                    );
                    continue;
                }
            };

            // 提取 AI 返回的内容
            let content = resp_json["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string();

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
                        "INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, raw_json, parsed_items, status, error_message, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'success', '', ?8)",
                        rusqlite::params![ocr_id, file_id, record_id_clone, project_id, checkup_date, content, parsed_items_str, now],
                    );

                        // 清理旧记录
                        let _: Result<usize, _> = conn.execute(
                        "DELETE FROM indicator_values WHERE ocr_result_id IN (SELECT id FROM ocr_results WHERE file_id = ?1 AND id != ?2)",
                        [&file_id, &ocr_id], 
                    );
                        let _: Result<usize, _> = conn.execute(
                            "DELETE FROM ocr_results WHERE file_id = ?1 AND id != ?2",
                            [&file_id, &ocr_id],
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
                                "INSERT INTO indicator_values (id, ocr_result_id, record_id, project_id, indicator_id, checkup_date, value, value_text, is_abnormal, created_at)
                                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                                rusqlite::params![
                                    iv_id, ocr_id, record_id_clone, project_id, indicator_id,
                                    checkup_date, value, item.value, item.is_abnormal as i32, now
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
                        "UPDATE checkup_records SET status = ?1, updated_at = ?2 WHERE id = ?3",
                        rusqlite::params![new_status, now, record_id_clone],
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
    app: tauri::AppHandle,
    db: tauri::State<'_, Database>,
    app_dir: tauri::State<'_, super::AppDir>,
) -> Result<String, String> {
    use tauri::Emitter;

    // 1. 查询必要信息
    let (file_info, record_id, checkup_date, config, model, ocr_prompt, indicators_map) =
        {
            let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
            let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

            // 查询 OCR 记录关联的信息
            let (file_id, record_id, project_id) = conn
                .query_row(
                    "SELECT file_id, record_id, project_id FROM ocr_results WHERE id = ?1",
                    [&ocr_id],
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
                    "SELECT checkup_date FROM checkup_records WHERE id = ?1",
                    [&record_id],
                    |row| row.get(0),
                )
                .unwrap_or_default();

            // 查询文件信息
            let (filename, stored_path, mime_type, project_name) = conn
                .query_row(
                    "SELECT f.original_filename, f.stored_path, f.mime_type, p.name
             FROM checkup_files f
             LEFT JOIN checkup_projects p ON f.project_id = p.id
             WHERE f.id = ?1",
                    [&file_id],
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

            let config = http_client::load_ai_config(&conn)?;
            let model = http_client::get_default_model(&conn);
            let ocr_prompt: String = conn.query_row(
            "SELECT config_value FROM system_config WHERE config_key = 'ocr_prompt_template'",
            [],
            |row| row.get(0),
        ).unwrap_or_else(|_| "请识别图片中的医疗检查报告...".to_string()); // Default shortened for brevity

            // 加载指标映射
            let mut ind_stmt = conn
                .prepare("SELECT id, project_id, name FROM indicators")
                .unwrap();
            let indicators: Vec<(String, String, String)> = ind_stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
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
                 "INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, raw_json, parsed_items, status, error_message, created_at)
                  VALUES (?1, ?2, ?3, ?4, ?5, '', '[]', 'processing', '', ?6)",
                 rusqlite::params![ocr_id_clone, file_id, record_id, project_id, checkup_date, now],
             );

                // 清理该文件的旧 OCR 记录 (只保留当前新创建的)
                // 1. 删除旧记录关联的指标值
                let _: Result<usize, _> = conn.execute(
                 "DELETE FROM indicator_values WHERE ocr_result_id IN (SELECT id FROM ocr_results WHERE file_id = ?1 AND id != ?2)",
                 [&file_id, &ocr_id_clone], 
             );
                // 2. 删除旧 OCR 记录
                let _: Result<usize, _> = conn.execute(
                    "DELETE FROM ocr_results WHERE file_id = ?1 AND id != ?2",
                    [&file_id, &ocr_id_clone],
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
            "max_tokens": 4096,
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
                update_ocr_failed(&app, &ocr_id_clone, &format!("请求失败: {}", e));
                return;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            update_ocr_failed(
                &app,
                &ocr_id_clone,
                &format!("API错误({}): {}", status, body),
            );
            return;
        }

        let resp_json: serde_json::Value = match response.json().await {
            Ok(v) => v,
            Err(e) => {
                update_ocr_failed(&app, &ocr_id_clone, &format!("解析响应失败: {}", e));
                return;
            }
        };

        let content = resp_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
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
                             "INSERT INTO indicator_values (id, ocr_result_id, record_id, project_id, indicator_id, checkup_date, value, value_text, is_abnormal, created_at)
                              VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                             rusqlite::params![iv_id, ocr_id_clone, record_id_clone, project_id, indicator_id, checkup_date, value, item.value, item.is_abnormal as i32, now],
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
    error_msg: &str,
) {
    if let Some(db_state) = app.try_state::<Database>() {
        if let Ok(conn_guard) = db_state.conn.lock() {
            if let Some(conn) = conn_guard.as_ref() {
                let ocr_id = uuid::Uuid::new_v4().to_string();
                let now = chrono::Local::now().to_rfc3339();
                let _: Result<usize, _> = conn.execute(
                "INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, raw_json, parsed_items, status, error_message, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, '', '[]', 'failed', ?6, ?7)",
                rusqlite::params![ocr_id, file_id, record_id, project_id, checkup_date, error_msg, now],
            );

                // 清理旧记录
                let _: Result<usize, _> = conn.execute(
                "DELETE FROM indicator_values WHERE ocr_result_id IN (SELECT id FROM ocr_results WHERE file_id = ?1 AND id != ?2)",
                [file_id, &ocr_id], 
            );
                let _: Result<usize, _> = conn.execute(
                    "DELETE FROM ocr_results WHERE file_id = ?1 AND id != ?2",
                    [file_id, &ocr_id],
                );
            }
        }
    }
}

/// 更新单个 OCR 指标项
#[tauri::command]
pub fn update_ocr_item(
    ocr_id: String,
    index: usize,
    item: OcrParsedItem,
    db: tauri::State<Database>,
) -> Result<(), String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    // 1. 获取当前数据
    let (parsed_items_str, project_id, record_id, checkup_date): (String, String, String, String) = conn.query_row(
        "SELECT parsed_items, project_id, record_id, checkup_date FROM ocr_results WHERE id = ?1",
        [&ocr_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    ).map_err(|e| format!("OCR记录不存在: {}", e))?;

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
        .prepare("SELECT id, project_id, name FROM indicators")
        .unwrap();
    let indicators_map: Vec<(String, String, String)> = ind_stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
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
                 "INSERT INTO indicator_values (id, ocr_result_id, record_id, project_id, indicator_id, checkup_date, value, value_text, is_abnormal, created_at)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                 rusqlite::params![iv_id, ocr_id, record_id, project_id, indicator_id, checkup_date, value, item.value, item.is_abnormal as i32, now],
             );
        }
    }

    Ok(())
}

/// 查询 OCR 状态
#[tauri::command]
pub fn get_ocr_status(
    record_id: String,
    db: tauri::State<Database>,
) -> Result<serde_json::Value, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    let total_files: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM checkup_files WHERE record_id = ?1",
            [&record_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    let total_ocr: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM ocr_results WHERE record_id = ?1",
            [&record_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    let success_ocr: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM ocr_results WHERE record_id = ?1 AND status = 'success'",
            [&record_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    let failed_ocr: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM ocr_results WHERE record_id = ?1 AND status = 'failed'",
            [&record_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    let record_status: String = conn
        .query_row(
            "SELECT status FROM checkup_records WHERE id = ?1",
            [&record_id],
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

/// 获取 OCR 结果
#[tauri::command]
pub fn get_ocr_results(
    record_id: String,
    db: tauri::State<Database>,
) -> Result<Vec<OcrResult>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, file_id, record_id, project_id, checkup_date, raw_json, parsed_items, status, error_message, created_at
             FROM ocr_results WHERE record_id = ?1
             ORDER BY created_at ASC"
        )
        .map_err(|e| format!("查询OCR结果失败: {}", e))?;

    let results = stmt
        .query_map([&record_id], |row| {
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
        })
        .map_err(|e| format!("查询失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析数据失败: {}", e))?;

    Ok(results)
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
