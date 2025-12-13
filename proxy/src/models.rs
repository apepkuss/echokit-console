use serde::{Deserialize, Serialize};

/// 设备状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceStatus {
    Online,
    Offline,
    Unknown,
}

impl std::fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceStatus::Online => write!(f, "online"),
            DeviceStatus::Offline => write!(f, "offline"),
            DeviceStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    /// 设备唯一标识符
    pub device_id: String,

    /// 设备名称
    pub name: String,

    /// MAC 地址
    pub mac_address: String,

    /// 绑定的容器 ID
    pub bound_container_id: Option<String>,

    /// 创建时间（Unix 时间戳）
    pub created_at: i64,

    /// 最后连接时间（Unix 时间戳）
    pub last_connected_at: Option<i64>,

    /// 设备状态
    pub status: DeviceStatus,

    /// 所属用户 ID
    pub user_id: Option<String>,
}

/// 容器信息
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    /// 容器 ID
    pub container_id: String,

    /// 容器名称
    pub name: String,

    /// EchoKit Server 主机地址（如：localhost、dallas.echokit.dev）
    pub host: String,

    /// EchoKit Server 端口
    pub port: u16,

    /// 协议类型（ws 或 wss）
    pub protocol: String,

    /// 容器状态
    pub status: String,
}

/// 健康检查响应
#[derive(Debug, Serialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub uptime_seconds: u64,
    pub active_connections: usize,
    pub database_connected: bool,
}
