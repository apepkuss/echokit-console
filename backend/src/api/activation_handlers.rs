//! 设备激活 API 处理器
//!
//! 实现设备通过 6 位激活码绑定到用户账号的流程

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use rand::Rng;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::models::{
    ActivationInfo, ApiError, AuthContext, ConfirmActivationRequest, ConfirmActivationResponse,
    GetActivationRequest, GetActivationResponse, VerifyActivationBoundResponse,
    VerifyActivationPendingResponse, VerifyActivationRequest,
};
use crate::store::{PgDeviceStore, RedisActivationStore};

/// 激活 API 状态
#[derive(Clone)]
pub struct ActivationState {
    pub activation_store: Arc<RedisActivationStore>,
    pub device_store: Arc<PgDeviceStore>,
    pub proxy_ws_url: String,
}

/// 生成 6 位数字激活码
fn generate_code() -> String {
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(0..1000000))
}

/// 生成 32 字节随机 challenge
fn generate_challenge() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    hex::encode(bytes)
}

/// 验证 device_id 格式（12 位小写十六进制）
fn is_valid_device_id(device_id: &str) -> bool {
    device_id.len() == 12 && device_id.chars().all(|c| c.is_ascii_hexdigit())
}

/// GET /api/activation - 设备请求激活码
pub async fn get_activation(
    Query(params): Query<GetActivationRequest>,
    State(state): State<ActivationState>,
) -> Result<Json<GetActivationResponse>, (StatusCode, Json<ApiError>)> {
    let device_id = params.device_id.to_lowercase();

    // 验证 device_id 格式
    if !is_valid_device_id(&device_id) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "invalid_device_id".to_string(),
                message: "device_id 格式错误，应为 12 位十六进制".to_string(),
            }),
        ));
    }

    // 检查是否已有未完成的激活（速率限制）
    match state.activation_store.has_pending_activation(&device_id).await {
        Ok(true) => {
            // 返回现有的激活码
            if let Ok(Some(info)) = state.activation_store.get_by_device(&device_id).await {
                if let Ok(Some(code)) = state.activation_store.get_code_by_device(&device_id).await
                {
                    info!("[Activation] 返回现有激活码: device={}", device_id);
                    return Ok(Json(GetActivationResponse {
                        code,
                        challenge: info.challenge,
                        expires_in: state.activation_store.default_ttl(),
                    }));
                }
            }
        }
        Ok(false) => {}
        Err(e) => {
            error!("[Activation] 检查激活状态失败: {}", e);
        }
    }

    // 生成新的激活码
    let code = generate_code();
    let challenge = generate_challenge();
    let now = Utc::now().timestamp();

    let info = ActivationInfo {
        device_id: device_id.clone(),
        challenge: challenge.clone(),
        confirmed_by: None,
        device_name: None,
        created_at: now,
    };

    // 存储到 Redis
    if let Err(e) = state.activation_store.create(&code, &info).await {
        error!("[Activation] 存储激活码失败: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "internal_error".to_string(),
                message: "生成激活码失败，请重试".to_string(),
            }),
        ));
    }

    info!(
        "[Activation] 生成激活码: device={}, code={}",
        device_id, code
    );

    Ok(Json(GetActivationResponse {
        code,
        challenge,
        expires_in: state.activation_store.default_ttl(),
    }))
}

/// POST /api/activation/confirm - 用户确认激活码
pub async fn confirm_activation(
    Extension(auth): Extension<AuthContext>,
    State(state): State<ActivationState>,
    Json(body): Json<ConfirmActivationRequest>,
) -> Result<Json<ConfirmActivationResponse>, (StatusCode, Json<ApiError>)> {
    // 验证激活码格式
    if body.code.len() != 6 || !body.code.chars().all(|c| c.is_ascii_digit()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "invalid_code_format".to_string(),
                message: "激活码格式错误，应为 6 位数字".to_string(),
            }),
        ));
    }

    // 查询激活码
    let mut info = match state.activation_store.get_by_code(&body.code).await {
        Ok(Some(info)) => info,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiError {
                    error: "code_not_found".to_string(),
                    message: "激活码不存在或已过期".to_string(),
                }),
            ));
        }
        Err(e) => {
            error!("[Activation] 查询激活码失败: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "internal_error".to_string(),
                    message: "查询激活码失败".to_string(),
                }),
            ));
        }
    };

    // 检查是否已被确认
    if info.confirmed_by.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiError {
                error: "already_confirmed".to_string(),
                message: "激活码已被确认".to_string(),
            }),
        ));
    }

    // 检查设备是否已绑定到其他用户
    if let Ok(Some((_, Some(owner)))) = state.device_store.get_device(&info.device_id).await {
        if owner != auth.user_id {
            return Err((
                StatusCode::CONFLICT,
                Json(ApiError {
                    error: "device_already_bound".to_string(),
                    message: "设备已绑定到其他用户".to_string(),
                }),
            ));
        }
    }

    // 更新激活信息
    info.confirmed_by = Some(auth.user_id.clone());
    info.device_name = body.device_name.clone();

    if let Err(e) = state.activation_store.update(&body.code, &info).await {
        error!("[Activation] 更新激活码失败: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "internal_error".to_string(),
                message: "确认激活码失败".to_string(),
            }),
        ));
    }

    info!(
        "[Activation] 用户确认激活码: user={}, device={}, code={}",
        auth.user_id, info.device_id, body.code
    );

    Ok(Json(ConfirmActivationResponse {
        status: "confirmed".to_string(),
        device_id: info.device_id,
    }))
}

/// POST /api/activation/verify - 设备验证激活状态
pub async fn verify_activation(
    State(state): State<ActivationState>,
    Json(body): Json<VerifyActivationRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    let device_id = body.device_id.to_lowercase();

    // 查询激活信息
    let info = match state.activation_store.get_by_device(&device_id).await {
        Ok(Some(info)) => info,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiError {
                    error: "activation_not_found".to_string(),
                    message: "未找到激活记录，可能已过期".to_string(),
                }),
            ));
        }
        Err(e) => {
            error!("[Activation] 查询激活信息失败: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "internal_error".to_string(),
                    message: "查询激活状态失败".to_string(),
                }),
            ));
        }
    };

    // 验证 challenge（确保是同一次激活流程）
    if info.challenge != body.challenge {
        warn!(
            "[Activation] Challenge 不匹配: device={}",
            device_id
        );
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                error: "invalid_challenge".to_string(),
                message: "Challenge 不匹配".to_string(),
            }),
        ));
    }

    // 检查是否已确认
    let user_id = match &info.confirmed_by {
        Some(id) => id.clone(),
        None => {
            // 未确认，返回 202 Accepted
            return Ok((
                StatusCode::ACCEPTED,
                Json(VerifyActivationPendingResponse {
                    status: "pending".to_string(),
                    retry_after_ms: 5000,
                }),
            )
                .into_response());
        }
    };

    // 创建设备记录
    let device_name = info
        .device_name
        .clone()
        .unwrap_or_else(|| format!("EchoKit-{}", &device_id[6..]));
    let firmware_version = &body.firmware_version;

    match state
        .device_store
        .create_device_for_user(&device_id, &device_name, &user_id, Some(firmware_version))
        .await
    {
        Ok(_) => {
            info!(
                "[Activation] 设备绑定成功: device={}, user={}, firmware={}",
                device_id, user_id, firmware_version
            );
        }
        Err(e) => {
            // 可能是设备已存在，尝试更新固件版本
            warn!(
                "[Activation] 创建设备记录失败（可能已存在）: device={}, error={}",
                device_id, e
            );
            // 尝试更新固件版本
            if let Err(e) = state
                .device_store
                .update_firmware_version(&device_id, firmware_version)
                .await
            {
                warn!(
                    "[Activation] 更新固件版本失败: device={}, error={}",
                    device_id, e
                );
            }
        }
    }

    // 删除激活记录
    if let Ok(Some(code)) = state.activation_store.get_code_by_device(&device_id).await {
        let _ = state.activation_store.delete(&code, &device_id).await;
    }

    // 返回成功
    Ok((
        StatusCode::OK,
        Json(VerifyActivationBoundResponse {
            status: "bound".to_string(),
            user_id,
            device_name,
            proxy_url: state.proxy_ws_url.clone(),
        }),
    )
        .into_response())
}
