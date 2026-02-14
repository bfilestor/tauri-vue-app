use axum::{
    extract::{Multipart, DefaultBodyLimit},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use qrcodegen::QrCode;
use qrcodegen::QrCodeEcc;
use std::net::SocketAddr;
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::oneshot;
use base64::Engine;

pub struct MobileServerState {
    pub shutdown_tx: Mutex<Option<oneshot::Sender<()>>>,
}

impl MobileServerState {
    pub fn new() -> Self {
        Self {
            shutdown_tx: Mutex::new(None),
        }
    }
}

pub fn init_state() -> MobileServerState {
    MobileServerState::new()
}

const HTML_CONTENT: &str = r#"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>上传体检报告</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; padding: 20px; text-align: center; background-color: #f5f7fa; color: #333; }
        .container { max-width: 600px; margin: 0 auto; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 12px 0 rgba(0,0,0,0.1); }
        h2 { color: #409EFF; margin-bottom: 20px; }
        .btn { display: inline-block; line-height: 1; white-space: nowrap; cursor: pointer; background: #fff; border: 1px solid #dcdfe6; color: #606266; -webkit-appearance: none; text-align: center; box-sizing: border-box; outline: none; margin: 0; transition: .1s; font-weight: 500; padding: 12px 20px; font-size: 14px; border-radius: 4px; width: 100%; margin-bottom: 15px; }
        .btn-primary { color: #fff; background-color: #409EFF; border-color: #409EFF; }
        .btn-success { color: #fff; background-color: #67C23A; border-color: #67C23A; }
        .file-list { margin-top: 10px; text-align: left; }
        .file-item { font-size: 14px; color: #606266; line-height: 1.8; border-bottom: 1px solid #ebeef5; padding: 5px 0; }
        .status { margin-top: 20px; font-weight: bold; }
        .success { color: #67C23A; }
        .error { color: #F56C6C; }
    </style>
</head>
<body>
    <div class="container">
        <h2>📷 上传体检报告</h2>
        <p style="color: #909399; font-size: 14px; margin-bottom: 20px;">请确保手机与电脑在同一 Wi-Fi 网络下</p>
        
        <input type="file" id="fileInput" multiple accept="image/*" style="display:none" onchange="handleFiles(this.files)">
        
        <button class="btn btn-primary" onclick="document.getElementById('fileInput').click()">
            选择图片 / 拍照
        </button>
        
        <div id="fileList" class="file-list"></div>
        
        <button class="btn btn-success" id="uploadBtn" style="display:none;" onclick="uploadFiles()">
            确认上传
        </button>
        
        <div id="status" class="status"></div>
    </div>

    <script>
        let selectedFiles = [];
        
        function handleFiles(files) {
            selectedFiles = Array.from(files);
            renderList();
        }
        
        function renderList() {
            const list = document.getElementById('fileList');
            list.innerHTML = '';
            selectedFiles.forEach(f => {
                const div = document.createElement('div');
                div.className = 'file-item';
                div.innerText = f.name + ' (' + (f.size/1024).toFixed(0) + 'KB)';
                list.appendChild(div);
            });
            const btn = document.getElementById('uploadBtn');
            if (selectedFiles.length > 0) {
                btn.style.display = 'inline-block';
                btn.innerText = '确认上传 (' + selectedFiles.length + '张)';
            } else {
                btn.style.display = 'none';
            }
            document.getElementById('status').innerText = '';
        }
        
        async function uploadFiles() {
            const status = document.getElementById('status');
            const btn = document.getElementById('uploadBtn');
            
            btn.disabled = true;
            btn.innerText = '上传中...';
            
            const formData = new FormData();
            selectedFiles.forEach(f => formData.append('files', f));
            
            try {
                const res = await fetch('/upload', { method: 'POST', body: formData });
                if (res.ok) {
                    status.innerHTML = '<span class="success">✅ 上传成功！请在电脑端查看。</span>';
                    selectedFiles = [];
                    renderList();
                } else {
                    status.innerHTML = '<span class="error">❌ 上传失败: ' + res.statusText + '</span>';
                }
            } catch (e) {
                status.innerHTML = '<span class="error">❌ 网络错误: ' + e.message + '</span>';
            } finally {
                btn.disabled = false;
            }
        }
    </script>
</body>
</html>
"#;

async fn index() -> Html<&'static str> {
    Html(HTML_CONTENT)
}

async fn upload(
    app_handle: AppHandle,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut count = 0;
    
    // 获取临时目录，用于暂存手机上传的文件
    let temp_dir = {

        let state = app_handle.state::<crate::commands::AppDir>();
        state.0.join("temp").join("mobile_uploads")
    };

    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        log::error!("Failed to create temp dir: {}", e);
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Server Error");
    }

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = if let Some(name) = field.file_name() {
            name.to_string()
        } else {
            let timestamp = chrono::Local::now().format("%H%M%S%f");
            format!("upload_{}.jpg", timestamp)
        };

        let file_path = temp_dir.join(&file_name);

        if let Ok(bytes) = field.bytes().await {
            if let Ok(_) = tokio::fs::write(&file_path, &bytes).await {
                count += 1;
                log::info!("Received mobile upload: {:?}", file_path);
                
                // 通知前端
                let _ = app_handle.emit("mobile_upload_success", serde_json::json!({
                    "filepath": file_path.to_string_lossy(),
                    "filename": file_name
                }));
            }
        }
    }

    if count > 0 {
        (axum::http::StatusCode::OK, "Upload successful")
    } else {
        (axum::http::StatusCode::BAD_REQUEST, "No files received")
    }
}

pub async fn start_server(app_handle: AppHandle) -> Result<(String, String), String> {
    let ip = local_ip_address::local_ip().map_err(|e| format!("无法获取本机IP: {}", e))?;
    
    // Bind to 0 port to get random available port
    let addr = SocketAddr::from((ip, 0));
    let listener = tokio::net::TcpListener::bind(addr).await
        .map_err(|e| format!("端口绑定失败: {}", e))?;
    
    let local_addr = listener.local_addr().map_err(|e| e.to_string())?;
    let port = local_addr.port();
    let url = format!("http://{}:{}/", ip, port);
    
    let app_handle_clone = app_handle.clone();
    let app = Router::new()
        .route("/", get(index))
        .route("/upload", post(move |multipart| upload(app_handle_clone, multipart)))
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024)); // 50MB limit

    let (tx, rx) = oneshot::channel();
    
    let state = app_handle.state::<MobileServerState>();
    let mut lock = state.shutdown_tx.lock().unwrap();
    if let Some(old_tx) = lock.take() {
        let _ = old_tx.send(());
    }
    *lock = Some(tx);
    drop(lock);

    let url_for_server = url.clone();
    tauri::async_runtime::spawn(async move {
        log::info!("Starting mobile server at {}", url_for_server);

        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                rx.await.ok();
            })
            .await 
        {
            log::error!("Server error: {}", e);
        }
        log::info!("Mobile server stopped");
    });

    // Generate QR Code
    let qr = QrCode::encode_text(&url, QrCodeEcc::Medium).map_err(|e| e.to_string())?;
    let svg = to_svg_string(&qr, 4);
    let base64_svg = base64::engine::general_purpose::STANDARD.encode(svg);
    let qr_data_uri = format!("data:image/svg+xml;base64,{}", base64_svg);

    Ok((url, qr_data_uri))
}

pub fn stop_server(state: tauri::State<MobileServerState>) {
    let mut lock = state.shutdown_tx.lock().unwrap();
    if let Some(tx) = lock.take() {
        let _ = tx.send(());
    }
}

// Helper to generate SVG string from QrCode
fn to_svg_string(qr: &QrCode, border: i32) -> String {
    let size = qr.size();
    let dim = size + border * 2;
    let mut res = String::with_capacity((dim as usize * dim as usize * 10) + 100);
    res.push_str(&format!(r##"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" viewBox="0 0 {0} {0}" stroke="none">"##, dim));
    res.push_str(r##"<rect width="100%" height="100%" fill="#FFFFFF"/>"##);
    res.push_str(r##"<path d=""##);
    for y in 0..size {
        for x in 0..size {
            if qr.get_module(x, y) {
                res.push_str(&format!("M{},{}h1v1h-1z ", x + border, y + border));
            }
        }
    }
    res.push_str(r##"" fill="#000000"/>"##);
    res.push_str("</svg>");
    res
}



