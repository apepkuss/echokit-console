use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use super::activation_handlers::{
    confirm_activation, get_activation, verify_activation, ActivationState,
};
use super::auth_handlers::{
    change_password, get_current_user, login, logout, register, update_current_user,
};
use super::device_handlers::{
    bind_device_to_server, delete_device, get_device, list_devices, register_device,
    report_device_info, unbind_device,
};
use super::handlers::{
    delete_container, deploy, get_container, get_container_health, get_container_logs,
    health_check, list_containers, start_container, stop_container,
};
use crate::docker::DockerManager;
use crate::middleware::auth_middleware;
use crate::store::{PgDeviceStore, PgUserStore, RedisActivationStore};

#[derive(Clone)]
pub struct AppState {
    pub docker_manager: Arc<DockerManager>,
    pub device_store: Arc<PgDeviceStore>,
    pub user_store: Arc<PgUserStore>,
    pub activation_store: Arc<RedisActivationStore>,
    pub proxy_ws_url: String,
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 认证路由（无需认证）
    let public_auth_routes = Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .with_state(state.user_store.clone());

    // 认证路由（需要认证）
    let protected_auth_routes = Router::new()
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(get_current_user))
        .route("/auth/me", put(update_current_user))
        .route("/auth/password", put(change_password))
        .layer(middleware::from_fn(auth_middleware))
        .with_state(state.user_store.clone());

    // 容器管理路由（需要认证）
    let container_routes = Router::new()
        .route("/deploy", post(deploy))
        .route("/containers", get(list_containers))
        .route("/containers/{id}", get(get_container))
        .route("/containers/{id}", delete(delete_container))
        .route("/containers/{id}/start", post(start_container))
        .route("/containers/{id}/stop", post(stop_container))
        .route("/containers/{id}/logs", get(get_container_logs))
        .route("/containers/{id}/health", get(get_container_health))
        .layer(middleware::from_fn(auth_middleware))
        .with_state(state.docker_manager.clone());

    // 设备管理路由（需要认证）
    let device_routes = Router::new()
        .route("/devices", get(list_devices))
        .route("/devices", post(register_device))
        .route("/devices/{id}", get(get_device))
        .route("/devices/{id}", delete(delete_device))
        .route("/devices/{id}/bind", post(bind_device_to_server))
        .route("/devices/{id}/unbind", post(unbind_device))
        .layer(middleware::from_fn(auth_middleware))
        .with_state(state.device_store.clone());

    // 设备上报路由（无需认证 - 设备调用）
    let public_device_routes = Router::new()
        .route("/devices/report", post(report_device_info))
        .with_state(state.device_store.clone());

    // 激活状态
    let activation_state = ActivationState {
        activation_store: state.activation_store,
        device_store: state.device_store,
        proxy_ws_url: state.proxy_ws_url,
    };

    // 激活路由（无需认证 - 设备请求）
    let public_activation_routes = Router::new()
        .route("/activation", get(get_activation))
        .route("/activation/verify", post(verify_activation))
        .with_state(activation_state.clone());

    // 激活路由（需要认证 - 用户确认）
    let protected_activation_routes = Router::new()
        .route("/activation/confirm", post(confirm_activation))
        .layer(middleware::from_fn(auth_middleware))
        .with_state(activation_state);

    let api_routes = Router::new()
        .merge(public_auth_routes)
        .merge(protected_auth_routes)
        .merge(container_routes)
        .merge(public_device_routes)
        .merge(device_routes)
        .merge(public_activation_routes)
        .merge(protected_activation_routes);

    Router::new()
        .route("/health", get(health_check))
        .nest("/api", api_routes)
        .layer(cors)
}
