//! HTTP 代理模块
//!
//! 将 /api/* 请求转发到 Backend 服务

use axum::{
    body::Body,
    extract::State,
    http::{header, Method, Response, StatusCode, Uri},
    response::IntoResponse,
};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

use crate::handler::AppState;

/// HTTP 代理处理器
///
/// 将 /api/* 请求转发到 Backend
pub async fn proxy_to_backend(
    State(state): State<Arc<AppState>>,
    method: Method,
    uri: Uri,
    headers: axum::http::HeaderMap,
    body: Body,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 提取路径和查询参数
    let path = uri.path();
    let query = uri.query().map(|q| format!("?{}", q)).unwrap_or_default();

    // 构建 Backend URL
    let backend_url = format!("{}{}{}", state.config.backend_url, path, query);

    info!(
        "[Proxy] HTTP 代理: {} {} -> {}",
        method, uri, backend_url
    );

    // 创建 HTTP 客户端（带超时，禁用系统代理）
    let timeout = Duration::from_millis(state.config.http_proxy_timeout_ms);
    let client = reqwest::Client::builder()
        .timeout(timeout)
        .no_proxy()  // 禁用系统代理，直连 Backend
        .build()
        .map_err(|e| {
            error!("[Proxy] 创建 HTTP 客户端失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("创建 HTTP 客户端失败: {}", e))
        })?;

    // 构建请求
    let mut backend_req = client.request(method.clone(), &backend_url);

    // 复制 headers（排除 Host 和 Connection）
    for (key, value) in headers.iter() {
        let key_str = key.as_str().to_lowercase();
        if key_str != "host" && key_str != "connection" {
            if let Ok(value_str) = value.to_str() {
                backend_req = backend_req.header(key.as_str(), value_str);
            }
        }
    }

    // 添加 X-Forwarded 头
    if let Some(host) = headers.get(header::HOST) {
        if let Ok(host_str) = host.to_str() {
            backend_req = backend_req.header("X-Forwarded-Host", host_str);
        }
    }
    backend_req = backend_req.header("X-Forwarded-Proto", "https");

    // 读取请求体
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|e| {
            error!("[Proxy] 读取请求体失败: {}", e);
            (StatusCode::BAD_REQUEST, format!("读取请求体失败: {}", e))
        })?;

    // 发送请求
    let response = backend_req
        .body(body_bytes)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                error!("[Proxy] Backend 请求超时");
                (StatusCode::GATEWAY_TIMEOUT, "Backend 请求超时".to_string())
            } else if e.is_connect() {
                error!("[Proxy] 无法连接到 Backend: {}", e);
                (StatusCode::BAD_GATEWAY, format!("无法连接到 Backend: {}", e))
            } else {
                error!("[Proxy] Backend 请求失败: {}", e);
                (StatusCode::BAD_GATEWAY, format!("Backend 请求失败: {}", e))
            }
        })?;

    // 转换响应
    let status = response.status();
    let resp_headers = response.headers().clone();

    info!(
        "[Proxy] HTTP 代理响应: {} {} -> {}",
        method, path, status
    );

    // 读取响应体
    let resp_body = response.bytes().await.map_err(|e| {
        error!("[Proxy] 读取响应体失败: {}", e);
        (StatusCode::BAD_GATEWAY, format!("读取响应体失败: {}", e))
    })?;

    // 构建响应
    let mut builder = Response::builder().status(status);

    // 复制响应头（排除 transfer-encoding，因为我们已经读取了完整的响应体）
    for (key, value) in resp_headers.iter() {
        let key_str = key.as_str().to_lowercase();
        if key_str != "transfer-encoding" {
            builder = builder.header(key, value);
        }
    }

    let response = builder
        .body(Body::from(resp_body))
        .map_err(|e| {
            error!("[Proxy] 构建响应失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("构建响应失败: {}", e))
        })?;

    Ok(response)
}
