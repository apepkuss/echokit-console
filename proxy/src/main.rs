mod config;
mod forwarder;
mod handler;
mod models;
mod store;

use std::future::IntoFuture;
use std::sync::Arc;

use anyhow::Context;
use axum::{
    routing::get,
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::ProxyConfig;
use crate::handler::{handle_device_websocket, health_check, AppState};
use crate::store::DeviceStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载环境变量
    dotenv::dotenv().ok();

    // 加载配置
    let config = ProxyConfig::from_env();

    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("echokit_proxy={},sqlx=warn", config.log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("启动 EchoKit Proxy 服务...");
    info!("配置信息:");
    info!("  - Proxy 端口: {}", config.proxy_port);
    info!("  - 健康检查端口: {}", config.health_check_port);
    info!("  - 数据库: {}", config.database_url.split('@').last().unwrap_or(""));
    info!("  - EchoKit Server 主机: {}", config.echokit_host);

    // 初始化数据库连接池
    info!("连接到数据库...");
    let pool = PgPoolOptions::new()
        .max_connections(config.db_pool_size)
        .connect(&config.database_url)
        .await
        .context("连接数据库失败")?;

    info!("数据库连接成功");

    // 初始化设备存储
    let device_store = DeviceStore::new(pool);

    // 创建应用状态
    let state = Arc::new(AppState {
        device_store,
        config: config.clone(),
    });

    // 创建 WebSocket 服务器路由
    let ws_app = Router::new()
        .route("/ws/{device_id}", get(handle_device_websocket))
        .with_state(state.clone())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // 创建健康检查服务器路由
    let health_app = Router::new()
        .route("/health", get(health_check))
        .with_state(state.clone());

    // 启动 WebSocket 服务器
    let ws_addr = format!("0.0.0.0:{}", config.proxy_port);
    let ws_listener = tokio::net::TcpListener::bind(&ws_addr).await?;
    info!("WebSocket 服务器监听: {}", ws_addr);

    // 启动健康检查服务器
    let health_addr = format!("0.0.0.0:{}", config.health_check_port);
    let health_listener = tokio::net::TcpListener::bind(&health_addr).await?;
    info!("健康检查服务器监听: {}", health_addr);

    info!("EchoKit Proxy 服务启动成功!");

    // 并发运行两个服务器
    tokio::try_join!(
        axum::serve(ws_listener, ws_app).into_future(),
        axum::serve(health_listener, health_app).into_future(),
    )?;

    Ok(())
}
