use std::env;

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// 数据库连接 URL
    pub database_url: String,

    /// Proxy WebSocket 服务端口
    pub proxy_port: u16,

    /// 健康检查 HTTP 端口
    pub health_check_port: u16,

    /// 日志级别
    pub log_level: String,

    /// WebSocket 超时时间（秒）
    pub ws_timeout: u64,

    /// 数据库连接池大小
    pub db_pool_size: u32,

    /// EchoKit Server 主机地址
    pub echokit_host: String,

    /// Backend API URL（用于 HTTP 代理）
    pub backend_url: String,

    /// HTTP 代理超时时间（毫秒）
    pub http_proxy_timeout_ms: u64,
}

impl ProxyConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://echokit:echokit@localhost:5432/echokit".to_string()),

            proxy_port: env::var("PROXY_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10086),

            health_check_port: env::var("HEALTH_CHECK_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10087),

            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),

            ws_timeout: env::var("WS_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),

            db_pool_size: env::var("DB_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),

            echokit_host: env::var("ECHOKIT_HOST")
                .unwrap_or_else(|_| "localhost".to_string()),

            backend_url: env::var("BACKEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),

            http_proxy_timeout_ms: env::var("HTTP_PROXY_TIMEOUT_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30000), // 默认 30 秒
        }
    }
}
