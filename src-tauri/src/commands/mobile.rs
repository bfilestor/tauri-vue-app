use crate::services::mobile_server;
use serde::Serialize;
use tauri::{AppHandle, State};

#[derive(Serialize)]
pub struct MobileConnectionInfo {
    pub url: String,
    pub qr_code: String,
}

#[tauri::command]
pub fn get_local_ips() -> Result<Vec<String>, String> {
    use local_ip_address::list_afinet_netifas;
    let network_interfaces = list_afinet_netifas().map_err(|e| format!("获取网卡信息失败: {}", e))?;
    let mut ips = Vec::new();
    for (_, ip) in network_interfaces {
        if ip.is_ipv4() && !ip.is_loopback() {
            ips.push(ip.to_string());
        }
    }
    if ips.is_empty() {
        if let Ok(ip) = local_ip_address::local_ip() {
            ips.push(ip.to_string());
        }
    }
    ips.sort();
    ips.dedup();
    Ok(ips)
}

#[tauri::command]
pub async fn start_mobile_server(app: AppHandle, selected_ip: Option<String>) -> Result<MobileConnectionInfo, String> {
    match mobile_server::start_server(app, selected_ip).await {
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
