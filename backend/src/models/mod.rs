use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 设备相关模型
mod device;
pub use device::*;

// 用户相关模型
mod user;
pub use user::*;

// 激活相关模型
mod activation;
pub use activation::*;

/// ASR 配置 - 根据平台类型区分
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "platform")]
pub enum ASRConfig {
    /// OpenAI ASR (Whisper)
    Openai {
        #[serde(rename = "apiKey")]
        api_key: String,
        model: String,
        lang: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        prompt: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },
    /// Paraformer ASR (阿里)
    Paraformer {
        #[serde(rename = "paraformerToken")]
        paraformer_token: String,
    },
}

/// LLM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMConfig {
    pub url: String,
    pub api_key: String,
    pub model: String,
    pub system_prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<u32>,
}

/// TTS 配置 - 根据平台类型区分
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "platform")]
pub enum TTSConfig {
    /// OpenAI TTS
    Openai {
        #[serde(rename = "apiKey")]
        api_key: String,
        model: String,
        voice: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },
    /// Groq TTS
    Groq {
        #[serde(rename = "apiKey")]
        api_key: String,
        model: String,
        voice: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },
    /// ElevenLabs TTS
    Elevenlabs {
        token: String,
        voice: String,
        #[serde(rename = "modelId", skip_serializing_if = "Option::is_none")]
        model_id: Option<String>,
        #[serde(rename = "languageCode", skip_serializing_if = "Option::is_none")]
        language_code: Option<String>,
    },
    /// GSV TTS (GPT-SoVITS)
    GSV {
        url: String,
        speaker: String,
        #[serde(rename = "apiKey", skip_serializing_if = "Option::is_none")]
        api_key: Option<String>,
        #[serde(rename = "timeoutSec", skip_serializing_if = "Option::is_none")]
        timeout_sec: Option<u32>,
    },
    /// StreamGSV TTS
    StreamGSV {
        url: String,
        speaker: String,
        #[serde(rename = "apiKey", skip_serializing_if = "Option::is_none")]
        api_key: Option<String>,
    },
    /// Fish TTS
    Fish {
        #[serde(rename = "apiKey")]
        api_key: String,
        speaker: String,
    },
    /// CosyVoice TTS (阿里百炼)
    CosyVoice {
        token: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        speaker: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
    },
}

/// EchoKit 完整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EchoKitConfig {
    pub name: String,
    pub asr: ASRConfig,
    pub llm: LLMConfig,
    pub tts: TTSConfig,
}

/// 部署请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployRequest {
    pub config: EchoKitConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

/// 容器状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContainerStatus {
    Running,
    Stopped,
    Error,
    Creating,
    Starting,
}

/// 健康状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

/// 健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub http_reachable: bool,
    pub container_running: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs_tail: Option<String>,
}

/// 部署响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployResponse {
    pub container_id: String,
    pub container_name: String,
    pub port: u16,
    pub ws_url: String,
    pub status: ContainerStatus,
    pub health: HealthCheckResult,
}

/// 容器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub port: u16,
    pub ws_url: String,
    pub status: ContainerStatus,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<HealthCheckResult>,
}

/// API 错误响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}
