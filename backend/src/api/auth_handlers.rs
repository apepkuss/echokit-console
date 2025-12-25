use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

use crate::middleware::generate_token;
use crate::models::{
    AuthContext, AuthResponse, ChangePasswordRequest, LoginRequest, RegisterRequest,
    UpdateUserRequest,
};
use crate::store::PgUserStore;

/// POST /auth/register - 用户注册
pub async fn register(
    State(user_store): State<Arc<PgUserStore>>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // 验证输入
    if let Err(errors) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "validation_error",
                "message": format!("{}", errors)
            })),
        )
            .into_response();
    }

    // 检查邮箱是否已存在
    match user_store.email_exists(&payload.email).await {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "email_exists",
                    "message": "Email already registered"
                })),
            )
                .into_response();
        }
        Ok(false) => {}
        Err(e) => {
            tracing::error!("Failed to check email existence: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to register user"
                })),
            )
                .into_response();
        }
    }

    // 创建用户
    match user_store
        .create(&payload.email, &payload.password, payload.name.as_deref())
        .await
    {
        Ok(user) => {
            // 生成 token
            match generate_token(&user.id, &user.email) {
                Ok(token) => (
                    StatusCode::CREATED,
                    Json(json!(AuthResponse { token, user })),
                )
                    .into_response(),
                Err(e) => {
                    tracing::error!("Failed to generate token: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "internal_error",
                            "message": "Failed to generate token"
                        })),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to create user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to register user"
                })),
            )
                .into_response()
        }
    }
}

/// POST /auth/login - 用户登录
pub async fn login(
    State(user_store): State<Arc<PgUserStore>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // 验证输入
    if let Err(errors) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "validation_error",
                "message": format!("{}", errors)
            })),
        )
            .into_response();
    }

    // 验证密码
    match user_store
        .verify_password(&payload.email, &payload.password)
        .await
    {
        Ok(Some(user)) => {
            // 生成 token
            match generate_token(&user.id, &user.email) {
                Ok(token) => (StatusCode::OK, Json(json!(AuthResponse { token, user }))).into_response(),
                Err(e) => {
                    tracing::error!("Failed to generate token: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "internal_error",
                            "message": "Failed to generate token"
                        })),
                    )
                        .into_response()
                }
            }
        }
        Ok(None) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "invalid_credentials",
                "message": "邮箱或密码错误"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to verify password: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to login"
                })),
            )
                .into_response()
        }
    }
}

/// POST /auth/logout - 用户登出（客户端清除 token 即可）
pub async fn logout() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "message": "Logged out successfully" })))
}

/// GET /auth/me - 获取当前用户信息
pub async fn get_current_user(
    State(user_store): State<Arc<PgUserStore>>,
    Extension(auth): Extension<AuthContext>,
) -> impl IntoResponse {
    match user_store.get_by_id(&auth.user_id).await {
        Ok(Some(user)) => (StatusCode::OK, Json(json!(user))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": "User not found"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to get user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to get user"
                })),
            )
                .into_response()
        }
    }
}

/// PUT /auth/me - 更新当前用户信息
pub async fn update_current_user(
    State(user_store): State<Arc<PgUserStore>>,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    match user_store.update(&auth.user_id, payload.name.as_deref()).await {
        Ok(user) => (StatusCode::OK, Json(json!(user))).into_response(),
        Err(e) => {
            tracing::error!("Failed to update user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to update user"
                })),
            )
                .into_response()
        }
    }
}

/// PUT /auth/password - 修改密码
pub async fn change_password(
    State(user_store): State<Arc<PgUserStore>>,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    // 验证输入
    if let Err(errors) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "validation_error",
                "message": format!("{}", errors)
            })),
        )
            .into_response();
    }

    match user_store
        .change_password(&auth.user_id, &payload.current_password, &payload.new_password)
        .await
    {
        Ok(true) => (
            StatusCode::OK,
            Json(json!({ "message": "Password changed successfully" })),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "invalid_password",
                "message": "Current password is incorrect"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to change password: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to change password"
                })),
            )
                .into_response()
        }
    }
}
