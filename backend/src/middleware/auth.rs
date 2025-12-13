use crate::models::{AuthContext, Claims};
use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_json::json;
use std::env;

/// JWT 密钥（从环境变量获取）
fn get_jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| "echokit-default-secret-change-in-production".into())
}

/// JWT 过期时间（秒），默认 7 天
fn get_jwt_expiration() -> i64 {
    env::var("JWT_EXPIRATION")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(7 * 24 * 60 * 60)
}

/// 生成 JWT Token
pub fn generate_token(user_id: &str, email: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now().timestamp();
    let exp = now + get_jwt_expiration();

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp,
        iat: now,
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(get_jwt_secret().as_bytes()),
    )
}

/// 验证 JWT Token
pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(get_jwt_secret().as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

/// 认证中间件
pub async fn auth_middleware(mut request: Request, next: Next) -> Response {
    // 从 Authorization header 获取 token
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "unauthorized",
                    "message": "Missing or invalid Authorization header"
                })),
            )
                .into_response();
        }
    };

    // 验证 token
    match verify_token(token) {
        Ok(claims) => {
            // 注入用户上下文
            let auth_context = AuthContext {
                user_id: claims.sub,
                email: claims.email,
            };
            request.extensions_mut().insert(auth_context);
            next.run(request).await
        }
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "unauthorized",
                "message": "Invalid or expired token"
            })),
        )
            .into_response(),
    }
}

/// 从请求中提取认证上下文
pub fn extract_auth_context(request: &Request<Body>) -> Option<AuthContext> {
    request.extensions().get::<AuthContext>().cloned()
}
