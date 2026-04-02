mod commands;
mod db;
mod services;

use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// 获取应用数据目录（程序所在目录）
fn get_app_data_dir(_app: &AppHandle) -> PathBuf {
    // 使用程序所在目录作为数据存储根目录
    let exe_path = std::env::current_exe().unwrap_or_default();
    exe_path
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf()
}

/// 确保 pictures 目录存在
fn ensure_pictures_dir(app_dir: &PathBuf) {
    let pictures_dir = app_dir.join("pictures");
    std::fs::create_dir_all(&pictures_dir).ok();
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
        "authorization" | "x-api-key" | "proxy-authorization" => {
            mask_sensitive_value(header_value)
        }
        _ => header_value.to_string(),
    }
}

fn log_request_debug(
    tag: &str,
    request: &reqwest::RequestBuilder,
    request_url: &str,
    request_body: &serde_json::Value,
) {
    log::info!("[{}] Request URL: {}", tag, request_url);
    log::info!("[{}] Request Body: {}", tag, request_body);

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
                log::warn!("[{}] Failed to build request for header logging: {}", tag, e);
            }
        }
    } else {
        log::warn!("[{}] RequestBuilder cannot be cloned for header logging", tag);
    }
}

fn log_response_debug(tag: &str, status: reqwest::StatusCode, response_body: &str) {
    log::info!("[{}] Response Status: {}", tag, status);
    log::info!("[{}] Response Body: {}", tag, response_body);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = show_window(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            quit,
            test_ai_connection,
            commands::config::get_config,
            commands::config::save_config,
            commands::project::list_projects,
            commands::project::create_project,
            commands::project::update_project,
            commands::project::delete_project,
            commands::indicator::list_indicators,
            commands::indicator::create_indicator,
            commands::indicator::update_indicator,
            commands::indicator::delete_indicator,
            commands::indicator::ensure_indicator,
            commands::record::list_records,
            commands::record::create_record,
            commands::record::update_record,
            commands::record::delete_record,
            commands::record::get_record,
            commands::record::get_or_create_today_record,
            commands::record::get_history_timeline,
            commands::file::upload_files,
            commands::file::list_files,
            commands::file::read_file_base64,
            commands::file::delete_file,
            commands::file::read_temp_file,

            commands::ocr::start_ocr,
            commands::ocr::retry_ocr,
            commands::ocr::get_ocr_status,
            commands::ocr::get_ocr_results,
            commands::ocr::update_ocr_item,
            commands::ai::start_ai_analysis,
            commands::ai::get_ai_analysis,
            commands::trend::get_project_trends,
            commands::trend::get_all_trends,
            commands::system::reset_checkup_data,
            commands::system::reset_all_data,
            commands::system::backup_data,
            commands::system::restore_data,
            commands::ai::get_ai_analyses_history,
            commands::ai::update_ai_analysis_content,
            commands::ai::chat_with_ai,
            commands::ai::get_chat_history,
            commands::ai::clear_chat_history,
            commands::mobile::get_local_ips,
            commands::mobile::start_mobile_server,
            commands::mobile::stop_mobile_server,
            commands::provider::list_providers,
            commands::provider::create_provider,
            commands::provider::update_provider,
            commands::provider::delete_provider,
            commands::provider::list_provider_models,
            commands::provider::add_model,
            commands::provider::update_model_info,
            commands::provider::delete_model,
            commands::provider::set_default_model,
            commands::provider::test_provider_connection,

        ])
        .setup(|app| {
            // 初始化日志（仅调试模式）
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // 初始化数据库
            let app_dir = get_app_data_dir(app.handle());
            ensure_pictures_dir(&app_dir);

            let database = db::Database::new(app_dir.clone()).expect("数据库初始化失败");

            // 将数据库实例和 app_dir 注入到 Tauri 状态
            app.manage(database);
            app.manage(commands::AppDir(app_dir));
            app.manage(services::mobile_server::init_state());

            // 旧数据迁移（ISS-054）
            {
                let database_state: tauri::State<db::Database> = app.state();
                let conn_guard = database_state.conn.lock().map_err(|e| e.to_string())?;
                if let Some(ref conn) = *conn_guard {
                    if let Err(e) = commands::provider::migrate_legacy_config(conn) {
                        log::warn!("旧版配置迁移失败: {}", e);
                    }
                }
            }



            log::info!("健康管家系统初始化完成");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_window(app: &AppHandle) {
    let windows = app.webview_windows();
    windows
        .values()
        .next()
        .expect("Sorry, no window found")
        .set_focus()
        .expect("Can't Bring Window to Focus");
}

#[tauri::command]
async fn test_ai_connection(db: tauri::State<'_, db::Database>) -> Result<String, String> {
    // 读取配置
    let (config, model) = {
        let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
        let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
        let config = services::http_client::load_ai_config(conn)?;
        let model = services::http_client::get_default_model(conn);
        (config, model)
    };

    // 构建 HTTP 客户端
    let client = services::http_client::build_client(&config)?;

    // 发送简单的测试请求
    let request_body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": "Hi, this is a connection test. Reply with 'OK' only."
            }
        ],
        "stream": true,
        "max_tokens": 10,
    });

    let request = client
        .post(&config.api_url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json");

    log_request_debug(
        "test_ai_connection",
        &request,
        &config.api_url,
        &request_body,
    );

    let response = request
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    let status = response.status();
    let response_body = response.text().await.unwrap_or_default();
    log_response_debug("test_ai_connection", status, &response_body);

    if status.is_success() {
        Ok(format!("连接成功！模型: {}", model))
    } else {
        Err(format!("API 返回错误 ({}): {}", status.as_u16(), response_body))
    }
}

#[tauri::command]
fn quit() {
    std::process::exit(0);
}
