use crate::models::{Device, DeviceStatus};
use anyhow::{Context, Result};
use sqlx::{PgPool, Row};

pub struct PgDeviceStore {
    pool: PgPool,
}

impl PgDeviceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取所有设备
    pub async fn list(&self) -> Result<Vec<Device>> {
        let rows = sqlx::query(
            r#"
            SELECT
                device_id,
                name,
                mac_address,
                bound_container_id,
                created_at,
                last_connected_at,
                status
            FROM devices
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch devices")?;

        let devices = rows
            .into_iter()
            .map(|row| {
                let status_str: String = row.get("status");
                let status = match status_str.as_str() {
                    "online" => DeviceStatus::Online,
                    "offline" => DeviceStatus::Offline,
                    _ => DeviceStatus::Unknown,
                };

                Device {
                    device_id: row.get("device_id"),
                    name: row.get("name"),
                    mac_address: row.get("mac_address"),
                    bound_container_id: row.get("bound_container_id"),
                    created_at: row.get("created_at"),
                    last_connected_at: row.get("last_connected_at"),
                    status,
                }
            })
            .collect();

        Ok(devices)
    }

    /// 获取单个设备
    pub async fn get(&self, device_id: &str) -> Result<Option<Device>> {
        let row = sqlx::query(
            r#"
            SELECT
                device_id,
                name,
                mac_address,
                bound_container_id,
                created_at,
                last_connected_at,
                status
            FROM devices
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch device")?;

        Ok(row.map(|row| {
            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "online" => DeviceStatus::Online,
                "offline" => DeviceStatus::Offline,
                _ => DeviceStatus::Unknown,
            };

            Device {
                device_id: row.get("device_id"),
                name: row.get("name"),
                mac_address: row.get("mac_address"),
                bound_container_id: row.get("bound_container_id"),
                created_at: row.get("created_at"),
                last_connected_at: row.get("last_connected_at"),
                status,
            }
        }))
    }

    /// 注册新设备
    pub async fn register(&self, device: Device) -> Result<Device> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO devices (
                device_id, name, mac_address, bound_container_id,
                created_at, last_connected_at, updated_at, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&device.device_id)
        .bind(&device.name)
        .bind(&device.mac_address)
        .bind(&device.bound_container_id)
        .bind(device.created_at)
        .bind(device.last_connected_at)
        .bind(now)
        .bind(device.status.to_string())
        .execute(&self.pool)
        .await
        .context("Failed to register device")?;

        Ok(device)
    }

    /// 更新设备
    pub async fn update(&self, device_id: &str, updates: Device) -> Result<Device> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            UPDATE devices
            SET
                name = $2,
                bound_container_id = $3,
                last_connected_at = $4,
                status = $5,
                updated_at = $6
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .bind(&updates.name)
        .bind(&updates.bound_container_id)
        .bind(updates.last_connected_at)
        .bind(updates.status.to_string())
        .bind(now)
        .execute(&self.pool)
        .await
        .context("Failed to update device")?;

        Ok(updates)
    }

    /// 删除设备
    pub async fn delete(&self, device_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM devices
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .execute(&self.pool)
        .await
        .context("Failed to delete device")?;

        Ok(())
    }

    /// 绑定设备到服务器
    pub async fn bind_to_server(&self, device_id: &str, container_id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            UPDATE devices
            SET
                bound_container_id = $2,
                updated_at = $3
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .bind(container_id)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("Failed to bind device to server")?;

        Ok(())
    }

    /// 解绑设备
    pub async fn unbind(&self, device_id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            UPDATE devices
            SET
                bound_container_id = NULL,
                updated_at = $2
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("Failed to unbind device")?;

        Ok(())
    }

    /// 获取容器的 WebSocket URL
    pub async fn get_container_ws_url(&self, container_id: &str) -> Result<Option<String>> {
        let row = sqlx::query(
            r#"
            SELECT name, host, port, use_tls
            FROM containers
            WHERE id = $1
            "#,
        )
        .bind(container_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch container")?;

        Ok(row.map(|row| {
            let name: String = row.get("name");
            let host: String = row.get("host");
            let port: Option<i32> = row.get("port");
            let use_tls: bool = row.get("use_tls");

            let protocol = if use_tls { "wss" } else { "ws" };
            let default_port = if use_tls { 443 } else { 80 };
            let port = port.unwrap_or(default_port);

            // 如果是标准端口，不显示端口号
            if (use_tls && port == 443) || (!use_tls && port == 80) {
                format!("{} ({}://{})", name, protocol, host)
            } else {
                format!("{} ({}://{}:{})", name, protocol, host, port)
            }
        }))
    }
}
