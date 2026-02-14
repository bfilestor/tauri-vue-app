use crate::services::mobile_server;
use serde::Serialize;
use tauri::{AppHandle, State};

#[derive(Serialize)]
pub struct MobileConnectionInfo {
    pub url: String,
    pub qr_code: String,
}

#[tauri::command]
pub async fn start_mobile_server(app: AppHandle) -> Result<MobileConnectionInfo, String> {
    match mobile_server::start_server(app).await {
        Ok((url, qr)) => Ok(MobileConnectionInfo {
            url,
            qr_code: qr,
        }),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn stop_mobile_server(state: State<'_, mobile_server::MobileServerState>) -> Result<(), String> {
    mobile_server::stop_server(state);
    Ok(())
}
