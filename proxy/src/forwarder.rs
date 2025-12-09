use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, WebSocketStream};
use tracing::{debug, error, info, warn};

type WsStream = WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// 双向转发 WebSocket 消息
///
/// 从设备到服务器，以及从服务器到设备
pub async fn bidirectional_forward(
    device_ws: axum::extract::ws::WebSocket,
    server_url: String,
    device_id: String,
) -> Result<()> {
    info!("开始双向转发: device_id={}, server_url={}", device_id, server_url);

    // 1. 连接到 EchoKit Server
    let (server_ws, _) = connect_async(&server_url)
        .await
        .context("连接到 EchoKit Server 失败")?;

    info!("已连接到 EchoKit Server: {}", server_url);

    // 2. 分离设备 WebSocket 的读写流
    let (mut device_tx, mut device_rx) = device_ws.split();

    // 3. 分离服务器 WebSocket 的读写流
    let (mut server_tx, mut server_rx) = server_ws.split();

    // 4. 创建两个转发任务

    // 设备 -> 服务器
    let device_to_server = async move {
        while let Some(msg) = device_rx.next().await {
            match msg {
                Ok(axum_msg) => {
                    // 转换 Axum WebSocket Message 到 tungstenite Message
                    let tungstenite_msg = match axum_msg {
                        axum::extract::ws::Message::Text(text) => {
                            debug!("设备->服务器 [Text]: {} bytes", text.len());
                            Message::Text(text.to_string().into())
                        }
                        axum::extract::ws::Message::Binary(data) => {
                            debug!("设备->服务器 [Binary]: {} bytes", data.len());
                            Message::Binary(data)
                        }
                        axum::extract::ws::Message::Ping(data) => {
                            debug!("设备->服务器 [Ping]");
                            Message::Ping(data)
                        }
                        axum::extract::ws::Message::Pong(data) => {
                            debug!("设备->服务器 [Pong]");
                            Message::Pong(data)
                        }
                        axum::extract::ws::Message::Close(frame) => {
                            info!("设备关闭连接");
                            if let Some(f) = frame {
                                Message::Close(Some(tokio_tungstenite::tungstenite::protocol::CloseFrame {
                                    code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::from(f.code),
                                    reason: f.reason.to_string().into(),
                                }))
                            } else {
                                Message::Close(None)
                            }
                        }
                    };

                    // 发送到服务器
                    if let Err(e) = server_tx.send(tungstenite_msg).await {
                        error!("发送消息到服务器失败: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("从设备接收消息失败: {}", e);
                    break;
                }
            }
        }

        info!("设备->服务器转发结束");
        Ok::<(), anyhow::Error>(())
    };

    // 服务器 -> 设备
    let server_to_device = async move {
        while let Some(msg) = server_rx.next().await {
            match msg {
                Ok(tungstenite_msg) => {
                    // 转换 tungstenite Message 到 Axum WebSocket Message
                    let axum_msg = match tungstenite_msg {
                        Message::Text(text) => {
                            debug!("服务器->设备 [Text]: {} bytes", text.len());
                            axum::extract::ws::Message::Text(text.to_string().into())
                        }
                        Message::Binary(data) => {
                            debug!("服务器->设备 [Binary]: {} bytes", data.len());
                            axum::extract::ws::Message::Binary(data)
                        }
                        Message::Ping(data) => {
                            debug!("服务器->设备 [Ping]");
                            axum::extract::ws::Message::Ping(data)
                        }
                        Message::Pong(data) => {
                            debug!("服务器->设备 [Pong]");
                            axum::extract::ws::Message::Pong(data)
                        }
                        Message::Close(frame) => {
                            info!("服务器关闭连接");
                            if let Some(f) = frame {
                                axum::extract::ws::Message::Close(Some(axum::extract::ws::CloseFrame {
                                    code: f.code.into(),
                                    reason: f.reason.to_string().into(),
                                }))
                            } else {
                                axum::extract::ws::Message::Close(None)
                            }
                        }
                        Message::Frame(_) => {
                            // 原始帧，通常不需要处理
                            continue;
                        }
                    };

                    // 发送到设备
                    if let Err(e) = device_tx.send(axum_msg).await {
                        error!("发送消息到设备失败: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("从服务器接收消息失败: {}", e);
                    break;
                }
            }
        }

        info!("服务器->设备转发结束");
        Ok::<(), anyhow::Error>(())
    };

    // 5. 并发运行两个转发任务
    let result = tokio::try_join!(device_to_server, server_to_device);

    match result {
        Ok(_) => {
            info!("双向转发正常结束: device_id={}", device_id);
            Ok(())
        }
        Err(e) => {
            warn!("双向转发异常结束: device_id={}, error={}", device_id, e);
            Err(e)
        }
    }
}
