use serde::{Deserialize, Serialize};
use sqlx::Type;

/// 设备状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DeviceStatus {
    Online,
    Offline,
    Unknown,
}

impl ToString for DeviceStatus {
    fn to_string(&self) -> String {
        match self {
            DeviceStatus::Online => "online".to_string(),
            DeviceStatus::Offline => "offline".to_string(),
            DeviceStatus::Unknown => "unknown".to_string(),
        }
    }
}

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub device_id: String,
    pub name: String,
    pub mac_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bound_container_id: Option<String>,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_connected_at: Option<i64>,
    pub status: DeviceStatus,
}

/// 设备注册请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterDeviceRequest {
    pub device_id: String,
    pub name: String,
    pub mac_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bound_container_id: Option<String>,
}

/// 绑定服务器请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindServerRequest {
    pub container_id: String,
}
