use serde::{Deserialize, Serialize};
use std::env;

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 服务器监听地址
    pub server_addr: String,
    /// 服务器监听端口
    pub server_port: u16,
    /// Docker 镜像名称
    pub docker_image: String,
    /// 配置文件存储目录
    pub config_dir: String,
    /// 录音存储目录
    pub record_dir: String,
    /// 默认 hello.wav 路径
    pub hello_wav_path: String,
    /// 容器端口范围起始
    pub port_range_start: u16,
    /// 容器端口范围结束
    pub port_range_end: u16,
    /// 外部访问地址（可选，用于替换 localhost）
    pub external_host: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_addr: "0.0.0.0".to_string(),
            server_port: 3000,
            docker_image: "secondstate/echokit:latest-server-vad".to_string(),
            config_dir: "./data/configs".to_string(),
            record_dir: "./data/records".to_string(),
            hello_wav_path: "./data/hello.wav".to_string(),
            port_range_start: 8080,
            port_range_end: 8180,
            external_host: None,
        }
    }
}

impl AppConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        Self {
            server_addr: env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000),
            docker_image: env::var("DOCKER_IMAGE")
                .unwrap_or_else(|_| "secondstate/echokit:latest-server-vad".to_string()),
            config_dir: env::var("CONFIG_DIR").unwrap_or_else(|_| "./data/configs".to_string()),
            record_dir: env::var("RECORD_DIR").unwrap_or_else(|_| "./data/records".to_string()),
            hello_wav_path: env::var("HELLO_WAV_PATH")
                .unwrap_or_else(|_| "./data/hello.wav".to_string()),
            port_range_start: env::var("PORT_RANGE_START")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8080),
            port_range_end: env::var("PORT_RANGE_END")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8180),
            external_host: env::var("EXTERNAL_HOST").ok(),
        }
    }

    /// 获取容器的 host 地址
    /// 如果设置了 EXTERNAL_HOST 则使用它，否则使用 localhost
    pub fn get_container_host(&self) -> &str {
        self.external_host.as_deref().unwrap_or("localhost")
    }
}
