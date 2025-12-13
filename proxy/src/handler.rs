use crate::config::ProxyConfig;
use crate::forwarder::bidirectional_forward;
use crate::store::DeviceStore;
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info, warn};

/// WebSocket 连接查询参数
///
/// 设备连接时会携带这些参数：
/// - 首次连接: `?opus=true`
/// - 重连（语音交互）: `?reconnect=true&opus=true`
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ConnectQueryParams {
    /// 是否为重连请求（语音交互时设备会重连）
    #[serde(default)]
    pub reconnect: bool,

    /// 是否启用 Opus 音频编码
    #[serde(default)]
    pub opus: bool,
}

#[derive(Clone)]
pub struct AppState {
    pub device_store: DeviceStore,
    pub config: ProxyConfig,
}

/// 标准化 device_id 格式（12位小写十六进制）
///
/// 设备发送的格式已经是标准格式（小写无冒号，如 "98a316f0b1e5"）
/// 数据库存储格式也统一为此格式
fn normalize_device_id(device_id: &str) -> String {
    // 如果是带冒号的格式，去掉冒号并转小写
    if device_id.contains(':') {
        return device_id.replace(":", "").to_lowercase();
    }

    // 其他格式直接转小写
    device_id.to_lowercase()
}

/// 构建查询参数字符串
///
/// 将 ConnectQueryParams 转换为 URL 查询字符串格式
/// 例如: "?reconnect=true&opus=true" 或 "?opus=true"
fn build_query_string(params: &ConnectQueryParams) -> String {
    let mut query_parts = Vec::new();

    if params.reconnect {
        query_parts.push("reconnect=true".to_string());
    }

    if params.opus {
        query_parts.push("opus=true".to_string());
    }

    if query_parts.is_empty() {
        String::new()
    } else {
        format!("?{}", query_parts.join("&"))
    }
}

/// 处理设备 WebSocket 连接请求
///
/// 路径: /ws/{device_id}?reconnect=false&opus=true
pub async fn handle_device_websocket(
    ws: WebSocketUpgrade,
    Path(device_id): Path<String>,
    Query(params): Query<ConnectQueryParams>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 标准化 device_id 格式（12位小写十六进制）
    let normalized_device_id = normalize_device_id(&device_id);
    info!(
        "[Proxy] 收到设备 WebSocket 连接请求: device_id={}, reconnect={}, opus={}",
        normalized_device_id, params.reconnect, params.opus
    );

    // 升级到 WebSocket 连接
    ws.on_upgrade(move |socket| handle_device_connection(socket, normalized_device_id, params, state))
}

/// 处理设备 WebSocket 连接
async fn handle_device_connection(
    device_ws: WebSocket,
    device_id: String,
    params: ConnectQueryParams,
    state: Arc<AppState>,
) {
    info!(
        "[Proxy] 设备 WebSocket 连接已建立: device_id={}, reconnect={}, opus={}",
        device_id, params.reconnect, params.opus
    );

    // 1. 查询设备信息
    let device = match state.device_store.get_device(&device_id).await {
        Ok(Some(device)) => device,
        Ok(None) => {
            error!("[Proxy] 设备不存在: device_id={}", device_id);
            return;
        }
        Err(e) => {
            error!("[Proxy] 查询设备失败: device_id={}, error={}", device_id, e);
            return;
        }
    };

    // 2. 检查设备是否绑定容器
    if device.bound_container_id.is_none() {
        warn!("[Proxy] 设备未绑定容器: device_id={}, device_name={}", device_id, device.name);
        return;
    }

    // 3. 查询容器信息
    let container = match state.device_store.get_container_for_device(&device_id).await {
        Ok(container) => container,
        Err(e) => {
            error!("[Proxy] 查询容器信息失败: device_id={}, error={}", device_id, e);
            return;
        }
    };

    // 4. 构建查询参数字符串
    let query_string = build_query_string(&params);

    // 5. 构建 EchoKit Server WebSocket URL（使用原始格式的 device_id + 查询参数）
    let server_url = if container.port == 443 || container.port == 80 {
        format!(
            "{}://{}/ws/{}{}",
            container.protocol, container.host, device_id, query_string
        )
    } else {
        format!(
            "{}://{}:{}/ws/{}{}",
            container.protocol, container.host, container.port, device_id, query_string
        )
    };

    // 用于日志的服务器 URL（不含 device_id 路径）
    let server_url_log = if container.port == 443 || container.port == 80 {
        format!("{}://{}", container.protocol, container.host)
    } else {
        format!("{}://{}:{}", container.protocol, container.host, container.port)
    };

    info!(
        "[Proxy] 路由设备到服务器: device_id={}, device_name={}, server={}, query={}",
        device_id, device.name, server_url_log, query_string
    );

    // 6. 标记设备为在线
    if let Err(e) = state.device_store.mark_device_online(&device_id).await {
        error!("[Proxy] 标记设备在线失败: device_id={}, error={}", device_id, e);
    }

    // 7. 开始双向转发
    info!(
        "[Proxy] 开始双向转发: device_id={} <-> server={}",
        device_id, server_url_log
    );
    match bidirectional_forward(device_ws, server_url, device_id.clone()).await {
        Ok(_) => {
            info!("[Proxy] 设备连接正常结束: device_id={}, server={}", device_id, server_url_log);
        }
        Err(e) => {
            error!("[Proxy] 设备连接异常结束: device_id={}, server={}, error={}", device_id, server_url_log, e);
        }
    }

    // 8. 标记设备为离线
    if let Err(e) = state.device_store.mark_device_offline(&device_id).await {
        error!("[Proxy] 标记设备离线失败: device_id={}, error={}", device_id, e);
    }

    info!("[Proxy] 设备 WebSocket 连接已关闭: device_id={}, server={}", device_id, server_url_log);
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
