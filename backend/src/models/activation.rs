//! 设备激活相关模型

use serde::{Deserialize, Serialize};

/// 激活码信息（存储在 Redis 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivationInfo {
    /// 设备 ID（12 位小写十六进制）
    pub device_id: String,
    /// 随机挑战字符串（64 字符十六进制）
    pub challenge: String,
    /// 确认用户 ID（用户确认后填充）
    pub confirmed_by: Option<String>,
    /// 确认后的设备名称
    pub device_name: Option<String>,
    /// 创建时间戳（Unix 秒）
    pub created_at: i64,
}

/// GET /api/activation 请求参数
///
/// 同时支持 `device_id` (snake_case) 和 `deviceId` (camelCase) 两种格式
#[derive(Debug, Deserialize)]
pub struct GetActivationRequest {
    /// 设备 ID（12 位小写十六进制）
    #[serde(alias = "deviceId")]
    pub device_id: String,
}

/// GET /api/activation 响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetActivationResponse {
    /// 6 位数字激活码
    pub code: String,
    /// 随机挑战字符串
    pub challenge: String,
    /// 激活码有效期（秒）
    pub expires_in: u64,
}

/// POST /api/activation/confirm 请求体
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmActivationRequest {
    /// 6 位数字激活码
    pub code: String,
    /// 设备名称（可选）
    pub device_name: Option<String>,
}

/// POST /api/activation/confirm 响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmActivationResponse {
    /// 状态
    pub status: String,
    /// 设备 ID
    pub device_id: String,
}

/// POST /api/activation/verify 请求体
///
/// 同时支持 snake_case 和 camelCase 两种格式
#[derive(Debug, Deserialize)]
pub struct VerifyActivationRequest {
    /// 设备 ID
    #[serde(alias = "deviceId")]
    pub device_id: String,
    /// 挑战字符串
    pub challenge: String,
    /// 固件版本
    #[serde(alias = "firmwareVersion")]
    pub firmware_version: String,
}

/// POST /api/activation/verify 响应 - 激活成功
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyActivationBoundResponse {
    /// 状态（bound）
    pub status: String,
    /// 用户 ID
    pub user_id: String,
    /// 设备名称
    pub device_name: String,
    /// Proxy WebSocket URL
    pub proxy_url: String,
}

/// POST /api/activation/verify 响应 - 等待确认
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyActivationPendingResponse {
    /// 状态（pending）
    pub status: String,
    /// 重试间隔（毫秒）
    pub retry_after_ms: u64,
}
