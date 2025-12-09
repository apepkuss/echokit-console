use crate::models::{ContainerInfo, Device, DeviceStatus};
use anyhow::{anyhow, Context, Result};
use sqlx::{PgPool, Row};
use tracing::{debug, error};

#[derive(Clone)]
pub struct DeviceStore {
    pool: PgPool,
}

impl DeviceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 根据设备 ID 获取设备信息
    pub async fn get_device(&self, device_id: &str) -> Result<Option<Device>> {
        debug!("查询设备信息: device_id={}", device_id);

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
        .context("查询设备失败")?;

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

    /// 获取设备绑定的容器信息
    pub async fn get_container_for_device(
        &self,
        device_id: &str,
    ) -> Result<ContainerInfo> {
        debug!("查询设备绑定的容器: device_id={}", device_id);

        // 1. 查询设备信息
        let device = self
            .get_device(device_id)
            .await?
            .ok_or_else(|| anyhow!("设备不存在: {}", device_id))?;

        // 2. 检查是否绑定容器
        let container_id = device
            .bound_container_id
            .ok_or_else(|| anyhow!("设备未绑定容器: {}", device_id))?;

        // 3. 根据容器 ID 判断是官方容器还是用户自建容器
        let (host, port, protocol) = self.resolve_container_endpoint(&container_id).await?;

        Ok(ContainerInfo {
            container_id: container_id.clone(),
            name: format!("echokit-server-{}", &container_id),
            host,
            port,
            protocol,
            status: "running".to_string(),
        })
    }

    /// 解析容器端点信息
    ///
    /// 从数据库查询容器的 host, port, use_tls 信息
    async fn resolve_container_endpoint(&self, container_id: &str) -> Result<(String, u16, String)> {
        debug!("解析容器端点: container_id={}", container_id);

        // 从数据库查询容器信息
        let row = sqlx::query(
            r#"
            SELECT host, port, use_tls
            FROM containers
            WHERE id = $1
            "#,
        )
        .bind(container_id)
        .fetch_optional(&self.pool)
        .await
        .context("查询容器信息失败")?
        .ok_or_else(|| anyhow!("容器不存在: {}", container_id))?;

        let host: String = row.get("host");
        let port: Option<i32> = row.get("port");
        let use_tls: bool = row.get("use_tls");

        // 如果 port 为 NULL，根据 use_tls 设置默认端口
        let port = port.map(|p| p as u16).unwrap_or(if use_tls { 443 } else { 80 });
        let protocol = if use_tls { "wss" } else { "ws" }.to_string();

        debug!(
            "容器端点: container_id={}, host={}, port={}, protocol={}",
            container_id, host, port, protocol
        );

        Ok((host, port, protocol))
    }

    /// 更新设备状态为在线
    pub async fn mark_device_online(&self, device_id: &str) -> Result<()> {
        debug!("标记设备在线: device_id={}", device_id);

        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            UPDATE devices
            SET
                status = 'online',
                last_connected_at = $2,
                updated_at = $2
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("更新设备状态失败")?;

        Ok(())
    }

    /// 更新设备状态为离线
    pub async fn mark_device_offline(&self, device_id: &str) -> Result<()> {
        debug!("标记设备离线: device_id={}", device_id);

        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            UPDATE devices
            SET
                status = 'offline',
                updated_at = $2
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("更新设备状态失败")?;

        Ok(())
    }

    /// 检查数据库连接是否正常
    pub async fn check_connection(&self) -> bool {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .is_ok()
    }
}
