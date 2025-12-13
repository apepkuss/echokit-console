use crate::models::User;
use anyhow::{anyhow, Context, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sqlx::{PgPool, Row};

pub struct PgUserStore {
    pool: PgPool,
}

impl PgUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 根据 ID 获取用户
    pub async fn get_by_id(&self, user_id: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, email, password_hash, name, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user by id")?;

        Ok(row.map(|row| User {
            id: row.get("id"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            name: row.get("name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    /// 根据邮箱获取用户
    pub async fn get_by_email(&self, email: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, email, password_hash, name, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user by email")?;

        Ok(row.map(|row| User {
            id: row.get("id"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            name: row.get("name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    /// 创建新用户
    pub async fn create(&self, email: &str, password: &str, name: Option<&str>) -> Result<User> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        // 密码哈希
        let password_hash = hash_password(password)?;

        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, name, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(&id)
        .bind(email)
        .bind(&password_hash)
        .bind(name)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("Failed to create user")?;

        Ok(User {
            id,
            email: email.to_string(),
            password_hash,
            name: name.map(String::from),
            created_at: now,
            updated_at: None,
        })
    }

    /// 验证用户密码
    pub async fn verify_password(&self, email: &str, password: &str) -> Result<Option<User>> {
        let user = self.get_by_email(email).await?;

        match user {
            Some(user) => {
                if verify_password(password, &user.password_hash)? {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// 更新用户信息
    pub async fn update(&self, user_id: &str, name: Option<&str>) -> Result<User> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            UPDATE users
            SET name = $2, updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(name)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("Failed to update user")?;

        self.get_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow!("User not found after update"))
    }

    /// 修改密码
    pub async fn change_password(
        &self,
        user_id: &str,
        current_password: &str,
        new_password: &str,
    ) -> Result<bool> {
        let user = self
            .get_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;

        // 验证当前密码
        if !verify_password(current_password, &user.password_hash)? {
            return Ok(false);
        }

        // 更新密码
        let new_hash = hash_password(new_password)?;
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $2, updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(&new_hash)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("Failed to change password")?;

        Ok(true)
    }

    /// 检查邮箱是否已存在
    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let row = sqlx::query(
            r#"
            SELECT 1 FROM users WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to check email existence")?;

        Ok(row.is_some())
    }
}

/// 密码哈希
fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("Failed to hash password: {}", e))?
        .to_string();

    Ok(password_hash)
}

/// 验证密码
fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| anyhow!("Failed to parse password hash: {}", e))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
