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

    /// 获取用户的所有设备
    pub async fn list(&self, user_id: &str) -> Result<Vec<Device>> {
        let rows = sqlx::query(
            r#"
            SELECT
                device_id,
                name,
                mac_address,
                bound_container_id,
                created_at,
                last_connected_at,
                status,
                firmware_version
            FROM devices
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
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
                    firmware_version: row.get("firmware_version"),
                }
            })
            .collect();

        Ok(devices)
    }

    /// 获取用户的单个设备
    pub async fn get(&self, device_id: &str, user_id: &str) -> Result<Option<Device>> {
        let row = sqlx::query(
            r#"
            SELECT
                device_id,
                name,
                mac_address,
                bound_container_id,
                created_at,
                last_connected_at,
                status,
                firmware_version
            FROM devices
            WHERE device_id = $1 AND user_id = $2
            "#,
        )
        .bind(device_id)
        .bind(user_id)
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
                firmware_version: row.get("firmware_version"),
            }
        }))
    }

    /// 注册新设备（关联到用户）
    pub async fn register(&self, device: Device, user_id: &str) -> Result<Device> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO devices (
                device_id, name, mac_address, bound_container_id,
                created_at, last_connected_at, updated_at, status, user_id, firmware_version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
        .bind(user_id)
        .bind(&device.firmware_version)
        .execute(&self.pool)
        .await
        .context("Failed to register device")?;

        Ok(device)
    }

    /// 更新用户的设备
    pub async fn update(&self, device_id: &str, user_id: &str, updates: Device) -> Result<Device> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            UPDATE devices
            SET
                name = $3,
                bound_container_id = $4,
                last_connected_at = $5,
                status = $6,
                updated_at = $7
            WHERE device_id = $1 AND user_id = $2
            "#,
        )
        .bind(device_id)
        .bind(user_id)
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

    /// 删除用户的设备
    pub async fn delete(&self, device_id: &str, user_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM devices
            WHERE device_id = $1 AND user_id = $2
            "#,
        )
        .bind(device_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .context("Failed to delete device")?;

        Ok(())
    }

    /// 绑定用户的设备到服务器
    pub async fn bind_to_server(&self, device_id: &str, user_id: &str, container_id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            UPDATE devices
            SET
                bound_container_id = $3,
                updated_at = $4
            WHERE device_id = $1 AND user_id = $2
            "#,
        )
        .bind(device_id)
        .bind(user_id)
        .bind(container_id)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("Failed to bind device to server")?;

        Ok(())
    }

    /// 解绑用户的设备
    pub async fn unbind(&self, device_id: &str, user_id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            UPDATE devices
            SET
                bound_container_id = NULL,
                updated_at = $3
            WHERE device_id = $1 AND user_id = $2
            "#,
        )
        .bind(device_id)
        .bind(user_id)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("Failed to unbind device")?;

        Ok(())
    }

    /// 获取设备信息（不检查用户，仅用于检查设备是否存在）
    pub async fn get_device(&self, device_id: &str) -> Result<Option<(Device, Option<String>)>> {
        let row = sqlx::query(
            r#"
            SELECT
                device_id,
                name,
                mac_address,
                bound_container_id,
                created_at,
                last_connected_at,
                status,
                user_id,
                firmware_version
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
            let user_id: Option<String> = row.get("user_id");
            let mac_address: Option<String> = row.get("mac_address");
            let device_id: String = row.get("device_id");

            (Device {
                device_id: device_id.clone(),
                name: row.get("name"),
                mac_address: mac_address.unwrap_or_else(|| device_id),
                bound_container_id: row.get("bound_container_id"),
                created_at: row.get("created_at"),
                last_connected_at: row.get("last_connected_at"),
                status,
                firmware_version: row.get("firmware_version"),
            }, user_id)
        }))
    }

    /// 为用户创建设备（激活时使用）
    ///
    /// device_id 和 mac_address 统一使用 12 位小写十六进制格式（如 "98a316f0b1e5"）
    pub async fn create_device_for_user(
        &self,
        device_id: &str,
        device_name: &str,
        user_id: &str,
        firmware_version: Option<&str>,
    ) -> Result<Device> {
        let now = chrono::Utc::now().timestamp();
        // device_id 和 mac_address 使用相同格式（12位小写十六进制）
        let normalized_device_id = device_id.to_lowercase();

        let device = Device {
            device_id: normalized_device_id.clone(),
            name: device_name.to_string(),
            mac_address: normalized_device_id,
            bound_container_id: None,
            created_at: now,
            last_connected_at: None,
            status: DeviceStatus::Offline,
            firmware_version: firmware_version.map(|v| v.to_string()),
        };

        sqlx::query(
            r#"
            INSERT INTO devices (
                device_id, name, mac_address, bound_container_id,
                created_at, last_connected_at, updated_at, status, user_id, firmware_version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
        .bind(user_id)
        .bind(&device.firmware_version)
        .execute(&self.pool)
        .await
        .context("Failed to create device for user")?;

        Ok(device)
    }

    /// 更新设备固件版本
    pub async fn update_firmware_version(
        &self,
        device_id: &str,
        firmware_version: &str,
    ) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            UPDATE devices
            SET firmware_version = $2, updated_at = $3
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .bind(firmware_version)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("Failed to update firmware version")?;

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
