mod db;
mod commands;
mod services;

use tauri::{AppHandle, Manager};
use std::path::PathBuf;

/// 获取应用数据目录（程序所在目录）
fn get_app_data_dir(_app: &AppHandle) -> PathBuf {
    // 使用程序所在目录作为数据存储根目录
    let exe_path = std::env::current_exe().unwrap_or_default();
    exe_path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf()
}

/// 确保 pictures 目录存在
fn ensure_pictures_dir(app_dir: &PathBuf) {
    let pictures_dir = app_dir.join("pictures");
    std::fs::create_dir_all(&pictures_dir).ok();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            commands::ocr::start_ocr,
            commands::ocr::get_ocr_status,
            commands::ocr::get_ocr_results,
            commands::ai::start_ai_analysis,
            commands::ai::get_ai_analysis,
            commands::trend::get_project_trends,
            commands::trend::get_all_trends,
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

            let database = db::Database::new(app_dir.clone())
                .expect("数据库初始化失败");

            // 将数据库实例和 app_dir 注入到 Tauri 状态
            app.manage(database);
            app.manage(commands::AppDir(app_dir));

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
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let config = services::http_client::load_ai_config(&conn)?;
        let model = services::http_client::get_default_model(&conn);
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
        "max_tokens": 10,
    });

    let response = client
        .post(&config.api_url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
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

#[tauri::command]
fn quit() {
    std::process::exit(0);
}