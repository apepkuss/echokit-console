use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info};

use crate::docker::DockerManager;
use crate::models::{ApiError, DeployRequest};

pub type AppState = Arc<DockerManager>;

/// 部署新的 EchoKit 实例
pub async fn deploy(
    State(manager): State<AppState>,
    Json(request): Json<DeployRequest>,
) -> impl IntoResponse {
    let instance_name = &request.config.name;
    let tts_platform = get_tts_platform_name(&request.config.tts);

    info!("========== 开始部署 EchoKit 实例 ==========");
    info!(
        "实例名称: {}, TTS平台: {}, 指定端口: {:?}",
        instance_name, tts_platform, request.port
    );

    let start_time = std::time::Instant::now();

    match manager.deploy(request.config.clone(), request.port).await {
        Ok(response) => {
            let elapsed = start_time.elapsed();
            let health_status = if response.health.status == crate::models::HealthStatus::Healthy {
                "✓ 健康"
            } else {
                "✗ 不健康"
            };

            info!("========== 部署完成 ==========");
            info!(
                "实例: {}, 容器ID: {}, 端口: {}, 状态: {}, 耗时: {:.2}s",
                response.container_name,
                &response.container_id[..12],
                response.port,
                health_status,
                elapsed.as_secs_f32()
            );
            info!("WebSocket地址: {}", response.ws_url);

            if response.health.status != crate::models::HealthStatus::Healthy {
                if let Some(ref err_msg) = response.health.error_message {
                    error!("健康检查失败: {}", err_msg);
                }
            }

            (StatusCode::OK, Json(serde_json::to_value(response).unwrap()))
        }
        Err(e) => {
            let elapsed = start_time.elapsed();
            let error_chain = format!("{:#}", e);

            error!("========== 部署失败 ==========");
            error!(
                "实例: {}, 耗时: {:.2}s, 错误: {}",
                instance_name,
                elapsed.as_secs_f32(),
                error_chain
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::to_value(ApiError {
                        error: "deploy_failed".to_string(),
                        message: error_chain,
                    })
                    .unwrap(),
                ),
            )
        }
    }
}

/// 获取 TTS 平台名称
fn get_tts_platform_name(tts: &crate::models::TTSConfig) -> &'static str {
    use crate::models::TTSConfig;
    match tts {
        TTSConfig::Openai { .. } => "OpenAI",
        TTSConfig::Groq { .. } => "Groq",
        TTSConfig::Elevenlabs { .. } => "ElevenLabs",
        TTSConfig::GSV { .. } => "GSV",
        TTSConfig::StreamGSV { .. } => "StreamGSV",
        TTSConfig::Fish { .. } => "Fish",
        TTSConfig::CosyVoice { .. } => "CosyVoice",
    }
}

/// 获取所有容器列表
pub async fn list_containers(State(manager): State<AppState>) -> impl IntoResponse {
    match manager.list_containers().await {
        Ok(containers) => (
            StatusCode::OK,
            Json(serde_json::to_value(containers).unwrap()),
        ),
        Err(e) => {
            let error_chain = format!("{:#}", e);
            error!("Failed to list containers: {}", error_chain);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::to_value(ApiError {
                        error: "list_failed".to_string(),
                        message: error_chain,
                    })
                    .unwrap(),
                ),
            )
        }
    }
}

/// 获取单个容器信息
pub async fn get_container(
    State(manager): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match manager.get_container(&id).await {
        Ok(container) => (
            StatusCode::OK,
            Json(serde_json::to_value(container).unwrap()),
        ),
        Err(e) => {
            let error_chain = format!("{:#}", e);
            error!("Failed to get container '{}': {}", id, error_chain);
            (
                StatusCode::NOT_FOUND,
                Json(
                    serde_json::to_value(ApiError {
                        error: "not_found".to_string(),
                        message: error_chain,
                    })
                    .unwrap(),
                ),
            )
        }
    }
}

/// 停止容器
pub async fn stop_container(
    State(manager): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("Stopping container: {}", id);
    match manager.stop_container(&id).await {
        Ok(()) => {
            info!("Container stopped: {}", id);
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            let error_chain = format!("{:#}", e);
            error!("Failed to stop container '{}': {}", id, error_chain);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::to_value(ApiError {
                        error: "stop_failed".to_string(),
                        message: error_chain,
                    })
                    .unwrap(),
                ),
            )
                .into_response()
        }
    }
}

/// 启动容器
pub async fn start_container(
    State(manager): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("Starting container: {}", id);
    match manager.start_container(&id).await {
        Ok(()) => {
            info!("Container started: {}", id);
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            let error_chain = format!("{:#}", e);
            error!("Failed to start container '{}': {}", id, error_chain);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::to_value(ApiError {
                        error: "start_failed".to_string(),
                        message: error_chain,
                    })
                    .unwrap(),
                ),
            )
                .into_response()
        }
    }
}

/// 删除容器
pub async fn delete_container(
    State(manager): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting container: {}", id);
    match manager.delete_container(&id).await {
        Ok(()) => {
            info!("Container deleted: {}", id);
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            let error_chain = format!("{:#}", e);
            error!("Failed to delete container '{}': {}", id, error_chain);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::to_value(ApiError {
                        error: "delete_failed".to_string(),
                        message: error_chain,
                    })
                    .unwrap(),
                ),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct LogsQuery {
    pub tail: Option<usize>,
}

/// 获取容器日志
pub async fn get_container_logs(
    State(manager): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<LogsQuery>,
) -> impl IntoResponse {
    match manager.get_container_logs(&id, query.tail).await {
        Ok(logs) => (StatusCode::OK, logs).into_response(),
        Err(e) => {
            let error_chain = format!("{:#}", e);
            error!("Failed to get logs for container '{}': {}", id, error_chain);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::to_value(ApiError {
                        error: "logs_failed".to_string(),
                        message: error_chain,
                    })
                    .unwrap(),
                ),
            )
                .into_response()
        }
    }
}

/// 健康检查（服务自身）
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}

/// 获取容器健康检查
pub async fn get_container_health(
    State(manager): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // 先获取容器信息
    match manager.get_container(&id).await {
        Ok(container) => {
            // 执行健康检查
            let health = manager.health_check(&container.id, container.port).await;
            (StatusCode::OK, Json(serde_json::to_value(health).unwrap()))
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(
                serde_json::to_value(ApiError {
                    error: "not_found".to_string(),
                    message: e.to_string(),
                })
                .unwrap(),
            ),
        ),
    }
}
