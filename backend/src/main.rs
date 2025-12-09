mod api;
mod config;
mod docker;
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
use crate::store::PgDeviceStore;

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

    // 初始化 Docker 管理器
    let docker_manager = DockerManager::new(config, pool.clone()).await?;

    // 初始化设备存储
    let device_store = PgDeviceStore::new(pool);

    // 创建应用状态
    let state = AppState {
        docker_manager: Arc::new(docker_manager),
        device_store: Arc::new(device_store),
    };

    // 创建路由
    let app = create_router(state);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
