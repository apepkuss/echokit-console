use crate::config::ProxyConfig;
use crate::forwarder::bidirectional_forward;
use crate::store::DeviceStore;
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct AppState {
    pub device_store: DeviceStore,
    pub config: ProxyConfig,
}

/// 标准化 MAC 地址格式（用于数据库查询）
///
/// 将设备发送的格式（小写无冒号，如 "98a316f0b1e5"）
/// 转换为数据库存储格式（大写带冒号，如 "98:A3:16:F0:B1:E4"）
fn normalize_mac_address(mac: &str) -> String {
    // 如果已经是带冒号的格式，直接返回大写版本
    if mac.contains(':') {
        return mac.to_uppercase();
    }

    // 如果是12位十六进制字符串（无冒号格式）
    if mac.len() == 12 && mac.chars().all(|c| c.is_ascii_hexdigit()) {
        let bytes: Vec<String> = (0..6)
            .map(|i| mac[i*2..i*2+2].to_uppercase())
            .collect();
        return bytes.join(":");
    }

    // 其他格式直接返回大写
    mac.to_uppercase()
}

/// 将 device_id 转换为日志友好格式（小写无冒号）
fn format_device_id_for_log(device_id: &str) -> String {
    device_id.replace(":", "").to_lowercase()
}

/// 处理设备 WebSocket 连接请求
///
/// 路径: /ws/{device_id}
pub async fn handle_device_websocket(
    ws: WebSocketUpgrade,
    Path(device_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let device_id_log = format_device_id_for_log(&device_id);
    info!("[Proxy] 收到设备 WebSocket 连接请求: device_id={}", device_id_log);

    // 升级到 WebSocket 连接
    ws.on_upgrade(move |socket| handle_device_connection(socket, device_id, state))
}

/// 处理设备 WebSocket 连接
async fn handle_device_connection(
    device_ws: WebSocket,
    device_id: String,
    state: Arc<AppState>,
) {
    // 用于日志的 device_id 格式（小写无冒号）
    let device_id_log = format_device_id_for_log(&device_id);

    info!("[Proxy] 设备 WebSocket 连接已建立: device_id={}", device_id_log);

    // 标准化 MAC 地址格式（用于数据库查询）
    let normalized_device_id = normalize_mac_address(&device_id);

    // 1. 查询设备信息
    let device = match state.device_store.get_device(&normalized_device_id).await {
        Ok(Some(device)) => device,
        Ok(None) => {
            error!("[Proxy] 设备不存在: device_id={}", device_id_log);
            return;
        }
        Err(e) => {
            error!("[Proxy] 查询设备失败: device_id={}, error={}", device_id_log, e);
            return;
        }
    };

    // 2. 检查设备是否绑定容器
    if device.bound_container_id.is_none() {
        warn!("[Proxy] 设备未绑定容器: device_id={}, device_name={}", device_id_log, device.name);
        return;
    }

    // 3. 查询容器信息
    let container = match state.device_store.get_container_for_device(&normalized_device_id).await {
        Ok(container) => container,
        Err(e) => {
            error!("[Proxy] 查询容器信息失败: device_id={}, error={}", device_id_log, e);
            return;
        }
    };

    // 4. 构建 EchoKit Server WebSocket URL（使用原始格式的 device_id）
    let server_url = if container.port == 443 || container.port == 80 {
        format!(
            "{}://{}/ws/{}",
            container.protocol, container.host, device_id
        )
    } else {
        format!(
            "{}://{}:{}/ws/{}",
            container.protocol, container.host, container.port, device_id
        )
    };

    // 用于日志的服务器 URL（不含 device_id 路径）
    let server_url_log = if container.port == 443 || container.port == 80 {
        format!("{}://{}", container.protocol, container.host)
    } else {
        format!("{}://{}:{}", container.protocol, container.host, container.port)
    };

    info!(
        "[Proxy] 路由设备到服务器: device_id={}, device_name={}, server={}",
        device_id_log, device.name, server_url_log
    );

    // 5. 标记设备为在线
    if let Err(e) = state.device_store.mark_device_online(&normalized_device_id).await {
        error!("[Proxy] 标记设备在线失败: device_id={}, error={}", device_id_log, e);
    }

    // 6. 开始双向转发
    info!(
        "[Proxy] 开始双向转发: device_id={} <-> server={}",
        device_id_log, server_url_log
    );
    match bidirectional_forward(device_ws, server_url, normalized_device_id.clone()).await {
        Ok(_) => {
            info!("[Proxy] 设备连接正常结束: device_id={}, server={}", device_id_log, server_url_log);
        }
        Err(e) => {
            error!("[Proxy] 设备连接异常结束: device_id={}, server={}, error={}", device_id_log, server_url_log, e);
        }
    }

    // 7. 标记设备为离线
    if let Err(e) = state.device_store.mark_device_offline(&normalized_device_id).await {
        error!("[Proxy] 标记设备离线失败: device_id={}, error={}", device_id_log, e);
    }

    info!("[Proxy] 设备 WebSocket 连接已关闭: device_id={}, server={}", device_id_log, server_url_log);
}

/// 健康检查接口
pub async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // 检查数据库连接
    let db_connected = state.device_store.check_connection().await;

    if db_connected {
        (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "status": "ok",
                "database_connected": true
            })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({
                "status": "error",
                "database_connected": false
            })),
        )
    }
}
