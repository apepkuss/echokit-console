mod api;
mod config;
mod docker;
mod middleware;
mod models;
mod store;

use std::sync::Arc;
use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::{create_router, router::AppState};
use crate::config::AppConfig;
use crate::docker::DockerManager;
use crate::store::{PgDeviceStore, PgUserStore, RedisActivationStore};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载环境变量
    dotenv::dotenv().ok();

    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "echokit_console=debug,tower_http=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let config = AppConfig::from_env();
    let addr = format!("{}:{}", config.server_addr, config.server_port);

    info!("Starting EchoKit Console server...");
    info!("Docker image: {}", config.docker_image);
    info!("Port range: {}-{}", config.port_range_start, config.port_range_end);

    // 初始化数据库连接池
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://echokit:echokit@localhost:5432/echokit".to_string());

    info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    info!("Database connected successfully");
    info!("Note: Run 'docker exec -i echokit-postgres psql -U echokit -d echokit < migrations/001_create_devices_table.sql' to initialize database");

    // 初始化 Redis 连接
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let activation_ttl = std::env::var("ACTIVATION_TTL_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(300u64); // 默认 5 分钟

    info!("Connecting to Redis: {}", redis_url);
    let activation_store = RedisActivationStore::new(&redis_url, activation_ttl)
        .context("Failed to connect to Redis")?;
    info!("Redis connected successfully (activation TTL: {}s)", activation_ttl);

    // 获取 Proxy WebSocket URL
    // 优先使用 PROXY_WS_URL，否则从 PROXY_EXTERNAL_HOST 和 PROXY_EXTERNAL_PORT 构建
    let proxy_ws_url = std::env::var("PROXY_WS_URL").unwrap_or_else(|_| {
        let host = std::env::var("PROXY_EXTERNAL_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("PROXY_EXTERNAL_PORT").unwrap_or_else(|_| "10086".to_string());
        format!("ws://{}:{}/ws", host, port)
    });
    info!("Proxy WebSocket URL: {}", proxy_ws_url);

    // 初始化 Docker 管理器
    let docker_manager = DockerManager::new(config, pool.clone()).await?;

    // 初始化设备存储
    let device_store = PgDeviceStore::new(pool.clone());

    // 初始化用户存储
    let user_store = PgUserStore::new(pool);

    // 创建应用状态
    let state = AppState {
        docker_manager: Arc::new(docker_manager),
        device_store: Arc::new(device_store),
        user_store: Arc::new(user_store),
        activation_store: Arc::new(activation_store),
        proxy_ws_url,
    };

    // 创建路由
    let app = create_router(state);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
