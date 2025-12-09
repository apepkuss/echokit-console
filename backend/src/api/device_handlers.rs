use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tracing::{error, info};

use crate::models::{
    ApiError, BindServerRequest, Device, DeviceStatus, RegisterDeviceRequest,
};
use crate::store::PgDeviceStore;

pub type DeviceStoreState = Arc<PgDeviceStore>;

/// 获取设备列表
pub async fn list_devices(State(store): State<DeviceStoreState>) -> impl IntoResponse {
    info!("获取设备列表");

    match store.list().await {
        Ok(devices) => {
            info!("成功获取 {} 个设备", devices.len());
            (StatusCode::OK, Json(devices))
        }
        Err(e) => {
            error!("获取设备列表失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(vec![] as Vec<Device>),
            )
        }
    }
}

/// 获取单个设备
pub async fn get_device(
    State(store): State<DeviceStoreState>,
    Path(device_id): Path<String>,
) -> impl IntoResponse {
    info!("获取设备: {}", device_id);

    match store.get(&device_id).await {
        Ok(Some(device)) => {
            info!("成功获取设备: {}", device.name);
            (StatusCode::OK, Json(Some(device))).into_response()
        }
        Ok(None) => {
            info!("设备不存在: {}", device_id);
            (
                StatusCode::NOT_FOUND,
                Json(ApiError {
                    error: "NotFound".to_string(),
                    message: format!("Device {} not found", device_id),
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("获取设备失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "InternalError".to_string(),
                    message: "Failed to fetch device".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// 注册新设备
pub async fn register_device(
    State(store): State<DeviceStoreState>,
    Json(request): Json<RegisterDeviceRequest>,
) -> impl IntoResponse {
    info!("注册新设备: {} ({})", request.name, request.mac_address);

    // 检查设备是否已存在
    if let Ok(Some(_)) = store.get(&request.device_id).await {
        info!("设备已注册: {}", request.device_id);
        return (
            StatusCode::CONFLICT,
            Json(ApiError {
                error: "AlreadyRegistered".to_string(),
                message: format!("设备 {} 已经注册过，无需重复注册", request.device_id),
            }),
        )
            .into_response();
    }

    let now = chrono::Utc::now().timestamp();
    let device = Device {
        device_id: request.device_id.clone(),
        name: request.name,
        mac_address: request.mac_address,
        bound_container_id: request.bound_container_id,
        created_at: now,
        last_connected_at: Some(now),
        status: DeviceStatus::Unknown,
    };

    match store.register(device.clone()).await {
        Ok(_) => {
            info!("设备注册成功: {}", device.device_id);
            (StatusCode::CREATED, Json(device)).into_response()
        }
        Err(e) => {
            error!("设备注册失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "InternalError".to_string(),
                    message: "Failed to register device".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// 删除设备
pub async fn delete_device(
    State(store): State<DeviceStoreState>,
    Path(device_id): Path<String>,
) -> impl IntoResponse {
    info!("删除设备: {}", device_id);

    // 检查设备是否存在
    if let Ok(None) = store.get(&device_id).await {
        info!("设备不存在: {}", device_id);
        return (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "NotFound".to_string(),
                message: format!("Device {} not found", device_id),
            }),
        )
            .into_response();
    }

    match store.delete(&device_id).await {
        Ok(_) => {
            info!("设备删除成功: {}", device_id);
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            error!("设备删除失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "InternalError".to_string(),
                    message: "Failed to delete device".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// 绑定设备到服务器
pub async fn bind_device_to_server(
    State(store): State<DeviceStoreState>,
    Path(device_id): Path<String>,
    Json(request): Json<BindServerRequest>,
) -> impl IntoResponse {
    // 将 device_id 转换为小写无冒号格式
    let device_id_normalized = device_id.replace(":", "").to_lowercase();

    // 获取设备当前信息（包括之前绑定的服务器）
    let device = match store.get(&device_id).await {
        Ok(Some(device)) => device,
        Ok(None) => {
            info!("[后端] 设备不存在: {}", device_id_normalized);
            return (
                StatusCode::NOT_FOUND,
                Json(ApiError {
                    error: "NotFound".to_string(),
                    message: format!("Device {} not found", device_id),
                }),
            )
                .into_response();
        }
        Err(e) => {
            error!("[后端] 查询设备失败: {}, 错误: {:?}", device_id_normalized, e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "InternalError".to_string(),
                    message: "Failed to get device".to_string(),
                }),
            )
                .into_response();
        }
    };

    // 获取原服务器的 WS URL
    let previous_server_url = if let Some(ref container_id) = device.bound_container_id {
        store
            .get_container_ws_url(container_id)
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "(未知)".to_string())
    } else {
        "(未绑定)".to_string()
    };

    // 获取目标服务器的 WS URL
    let target_server_url = store
        .get_container_ws_url(&request.container_id)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| request.container_id.clone());

    info!(
        "[后端] 切换服务器请求: device_id={}, device_name={}, 原服务器={}, 目标服务器={}",
        device_id_normalized, device.name, previous_server_url, target_server_url
    );

    match store
        .bind_to_server(&device_id, &request.container_id)
        .await
    {
        Ok(_) => {
            info!(
                "[后端] 切换服务器成功: device_id={}, 原服务器={} -> 新服务器={}",
                device_id_normalized, previous_server_url, target_server_url
            );
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            error!(
                "[后端] 切换服务器失败: device_id={}, 目标服务器={}, 错误={:?}",
                device_id_normalized, target_server_url, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "InternalError".to_string(),
                    message: "Failed to bind device".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// 解绑设备
pub async fn unbind_device(
    State(store): State<DeviceStoreState>,
    Path(device_id): Path<String>,
) -> impl IntoResponse {
    // 将 device_id 转换为小写无冒号格式
    let device_id_normalized = device_id.replace(":", "").to_lowercase();

    // 获取设备当前信息（包括之前绑定的服务器）
    let device = match store.get(&device_id).await {
        Ok(Some(device)) => device,
        Ok(None) => {
            info!("[后端] 设备不存在: {}", device_id_normalized);
            return (
                StatusCode::NOT_FOUND,
                Json(ApiError {
                    error: "NotFound".to_string(),
                    message: format!("Device {} not found", device_id),
                }),
            )
                .into_response();
        }
        Err(e) => {
            error!("[后端] 查询设备失败: {}, 错误: {:?}", device_id_normalized, e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "InternalError".to_string(),
                    message: "Failed to get device".to_string(),
                }),
            )
                .into_response();
        }
    };

    // 获取原服务器的 WS URL
    let previous_server_url = if let Some(ref container_id) = device.bound_container_id {
        store
            .get_container_ws_url(container_id)
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "(未知)".to_string())
    } else {
        "(未绑定)".to_string()
    };

    info!(
        "[后端] 解绑服务器请求: device_id={}, device_name={}, 原服务器={}",
        device_id_normalized, device.name, previous_server_url
    );

    match store.unbind(&device_id).await {
        Ok(_) => {
            info!(
                "[后端] 解绑服务器成功: device_id={}, 已解除与 {} 的绑定",
                device_id_normalized, previous_server_url
            );
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            error!(
                "[后端] 解绑服务器失败: device_id={}, 错误={:?}",
                device_id_normalized, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "InternalError".to_string(),
                    message: "Failed to unbind device".to_string(),
                }),
            )
                .into_response()
        }
    }
}
