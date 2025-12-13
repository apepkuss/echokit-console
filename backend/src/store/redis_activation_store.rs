//! Redis 激活码存储
//!
//! 使用 Redis 存储设备激活码信息，支持 TTL 自动过期

use anyhow::Result;
use redis::AsyncCommands;
use tracing::{debug, error};

use crate::models::ActivationInfo;

/// Redis 激活码存储
pub struct RedisActivationStore {
    client: redis::Client,
    default_ttl: u64,
}

impl RedisActivationStore {
    /// 创建新的 Redis 激活码存储
    pub fn new(redis_url: &str, default_ttl: u64) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        debug!("[RedisActivationStore] 已连接到 Redis: {}", redis_url);
        Ok(Self { client, default_ttl })
    }

    /// 创建激活码
    pub async fn create(&self, code: &str, info: &ActivationInfo) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let key = format!("activation:{}", code);
        let device_key = format!("activation:device:{}", info.device_id);
        let json = serde_json::to_string(info)?;

        // 存储激活信息
        conn.set_ex::<_, _, ()>(&key, &json, self.default_ttl).await?;

        // 存储反向索引（device_id -> code）
        conn.set_ex::<_, _, ()>(&device_key, code, self.default_ttl).await?;

        debug!(
            "[RedisActivationStore] 创建激活码: code={}, device_id={}, ttl={}s",
            code, info.device_id, self.default_ttl
        );
        Ok(())
    }

    /// 根据激活码查询
    pub async fn get_by_code(&self, code: &str) -> Result<Option<ActivationInfo>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("activation:{}", code);

        let json: Option<String> = conn.get(&key).await?;
        match json {
            Some(j) => {
                let info: ActivationInfo = serde_json::from_str(&j)?;
                debug!(
                    "[RedisActivationStore] 查询激活码: code={}, device_id={}",
                    code, info.device_id
                );
                Ok(Some(info))
            }
            None => {
                debug!("[RedisActivationStore] 激活码不存在: code={}", code);
                Ok(None)
            }
        }
    }

    /// 根据设备 ID 查询激活码
    pub async fn get_code_by_device(&self, device_id: &str) -> Result<Option<String>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let device_key = format!("activation:device:{}", device_id);

        let code: Option<String> = conn.get(&device_key).await?;
        debug!(
            "[RedisActivationStore] 根据设备查询激活码: device_id={}, code={:?}",
            device_id, code
        );
        Ok(code)
    }

    /// 根据设备 ID 查询激活信息
    pub async fn get_by_device(&self, device_id: &str) -> Result<Option<ActivationInfo>> {
        if let Some(code) = self.get_code_by_device(device_id).await? {
            self.get_by_code(&code).await
        } else {
            Ok(None)
        }
    }

    /// 更新激活信息（确认时使用）
    pub async fn update(&self, code: &str, info: &ActivationInfo) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("activation:{}", code);

        // 获取剩余 TTL
        let ttl: i64 = redis::cmd("TTL")
            .arg(&key)
            .query_async(&mut conn)
            .await?;

        if ttl <= 0 {
            error!("[RedisActivationStore] 激活码已过期: code={}", code);
            return Err(anyhow::anyhow!("激活码已过期"));
        }

        // 更新数据
        let json = serde_json::to_string(info)?;
        conn.set_ex::<_, _, ()>(&key, &json, ttl as u64).await?;

        debug!(
            "[RedisActivationStore] 更新激活信息: code={}, confirmed_by={:?}, ttl={}s",
            code, info.confirmed_by, ttl
        );
        Ok(())
    }

    /// 删除激活记录
    pub async fn delete(&self, code: &str, device_id: &str) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let key = format!("activation:{}", code);
        let device_key = format!("activation:device:{}", device_id);

        conn.del::<_, ()>(&[&key, &device_key]).await?;
        debug!(
            "[RedisActivationStore] 删除激活记录: code={}, device_id={}",
            code, device_id
        );
        Ok(())
    }

    /// 检查设备是否有未完成的激活（速率限制）
    pub async fn has_pending_activation(&self, device_id: &str) -> Result<bool> {
        Ok(self.get_code_by_device(device_id).await?.is_some())
    }

    /// 获取默认 TTL
    pub fn default_ttl(&self) -> u64 {
        self.default_ttl
    }
}
