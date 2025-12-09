use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use super::device_handlers::{
    bind_device_to_server, delete_device, get_device, list_devices, register_device,
    unbind_device,
};
use super::handlers::{
    delete_container, deploy, get_container, get_container_health, get_container_logs,
    health_check, list_containers, start_container, stop_container,
};
use crate::docker::DockerManager;
use crate::store::PgDeviceStore;

#[derive(Clone)]
pub struct AppState {
    pub docker_manager: Arc<DockerManager>,
    pub device_store: Arc<PgDeviceStore>,
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 容器管理路由
    let container_routes = Router::new()
        .route("/deploy", post(deploy))
        .route("/containers", get(list_containers))
        .route("/containers/{id}", get(get_container))
        .route("/containers/{id}", delete(delete_container))
        .route("/containers/{id}/start", post(start_container))
        .route("/containers/{id}/stop", post(stop_container))
        .route("/containers/{id}/logs", get(get_container_logs))
        .route("/containers/{id}/health", get(get_container_health))
        .with_state(state.docker_manager.clone());

    // 设备管理路由
    let device_routes = Router::new()
        .route("/devices", get(list_devices))
        .route("/devices", post(register_device))
        .route("/devices/{id}", get(get_device))
        .route("/devices/{id}", delete(delete_device))
        .route("/devices/{id}/bind", post(bind_device_to_server))
        .route("/devices/{id}/unbind", post(unbind_device))
        .with_state(state.device_store);

    let api_routes = Router::new()
        .merge(container_routes)
        .merge(device_routes);

    Router::new()
        .route("/health", get(health_check))
        .nest("/api", api_routes)
        .layer(cors)
}
