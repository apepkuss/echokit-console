use anyhow::{Context, Result};
use bollard::models::{ContainerSummaryStateEnum, HostConfig, PortBinding};
use bollard::query_parameters::{
    CreateContainerOptions, CreateImageOptions, InspectContainerOptions, ListContainersOptions,
    LogsOptions, RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
};
use bollard::secret::ContainerCreateBody;
use bollard::Docker;
use futures_util::TryStreamExt;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::AppConfig;
use crate::models::{
    ContainerInfo, ContainerStatus, DeployResponse, EchoKitConfig, HealthCheckResult, HealthStatus,
};

use super::generate_config_toml;

/// 从容器日志中提取错误提示
fn extract_error_hint(logs: &str) -> Option<String> {
    // 常见错误模式
    let error_patterns = [
        ("TOML parse error", "Configuration file (config.toml) has invalid TOML syntax"),
        ("data did not match any variant", "Configuration format mismatch - check TTS/ASR/LLM settings"),
        ("missing field", "Missing required configuration field"),
        ("unknown field", "Unknown configuration field - check spelling"),
        ("Address already in use", "Port 8080 is already in use inside the container"),
        ("Connection refused", "Cannot connect to external service - check API endpoints"),
        ("No such file or directory", "Required file not found"),
        ("Permission denied", "Permission error - check file permissions"),
        ("panicked at", "Application crashed - check configuration"),
    ];

    for (pattern, hint) in error_patterns {
        if logs.contains(pattern) {
            // 尝试提取更具体的错误信息
            if let Some(line) = logs.lines().find(|l| l.contains(pattern)) {
                return Some(format!("{}: {}", hint, line.trim()));
            }
            return Some(hint.to_string());
        }
    }

    // 查找 "error" 关键词
    if let Some(line) = logs.lines().find(|l| l.to_lowercase().contains("error")) {
        return Some(line.trim().to_string());
    }

    None
}

/// 健康检查配置
const HEALTH_CHECK_TIMEOUT_SECS: u64 = 5;
const HEALTH_CHECK_RETRIES: u32 = 3;
const HEALTH_CHECK_RETRY_DELAY_MS: u64 = 1000;

/// Docker 容器管理器
pub struct DockerManager {
    docker: Docker,
    config: AppConfig,
    used_ports: Arc<RwLock<Vec<u16>>>,
    http_client: reqwest::Client,
    pool: sqlx::PgPool,
}

impl DockerManager {
    /// 创建新的 Docker 管理器
    pub async fn new(config: AppConfig, pool: sqlx::PgPool) -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()
            .context("Failed to connect to Docker daemon")?;

        // 确保目录存在
        fs::create_dir_all(&config.config_dir).await?;
        fs::create_dir_all(&config.record_dir).await?;

        // 创建 HTTP 客户端用于健康检查
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(HEALTH_CHECK_TIMEOUT_SECS))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            docker,
            config,
            used_ports: Arc::new(RwLock::new(Vec::new())),
            http_client,
            pool,
        })
    }

    /// 分配可用端口
    async fn allocate_port(&self) -> Result<u16> {
        let mut used_ports = self.used_ports.write().await;

        // 获取已使用的端口
        let containers = self.list_containers().await?;
        for container in &containers {
            if !used_ports.contains(&container.port) {
                used_ports.push(container.port);
            }
        }

        // 查找可用端口
        for port in self.config.port_range_start..=self.config.port_range_end {
            if !used_ports.contains(&port) {
                used_ports.push(port);
                return Ok(port);
            }
        }

        anyhow::bail!("No available ports in range")
    }

    /// 检查容器是否在运行
    async fn is_container_running(&self, container_id: &str) -> bool {
        match self
            .docker
            .inspect_container(container_id, None::<InspectContainerOptions>)
            .await
        {
            Ok(info) => info
                .state
                .and_then(|s| s.running)
                .unwrap_or(false),
            Err(_) => false,
        }
    }

    /// 执行 HTTP 健康检查
    async fn check_http_health(&self, port: u16) -> bool {
        let url = format!("http://localhost:{}/", port);
        match self.http_client.get(&url).send().await {
            // 只要能收到响应就认为服务可用（即使是 404 也说明服务在运行）
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// 执行完整的健康检查
    pub async fn health_check(&self, container_id: &str, port: u16) -> HealthCheckResult {
        // 检查容器是否在运行
        let container_running = self.is_container_running(container_id).await;

        if !container_running {
            // 容器未运行，获取错误日志
            let logs = self.get_container_logs(container_id, Some(50)).await.ok();
            return HealthCheckResult {
                status: HealthStatus::Unhealthy,
                http_reachable: false,
                container_running: false,
                error_message: Some("Container is not running".to_string()),
                logs_tail: logs,
            };
        }

        // 容器运行中，检查 HTTP 可达性（带重试）
        let mut http_reachable = false;
        for attempt in 1..=HEALTH_CHECK_RETRIES {
            if self.check_http_health(port).await {
                http_reachable = true;
                break;
            }
            if attempt < HEALTH_CHECK_RETRIES {
                tokio::time::sleep(Duration::from_millis(HEALTH_CHECK_RETRY_DELAY_MS)).await;
            }
        }

        if http_reachable {
            HealthCheckResult {
                status: HealthStatus::Healthy,
                http_reachable: true,
                container_running: true,
                error_message: None,
                logs_tail: None,
            }
        } else {
            // HTTP 不可达，获取日志帮助诊断
            let logs = self.get_container_logs(container_id, Some(50)).await.ok();
            HealthCheckResult {
                status: HealthStatus::Unhealthy,
                http_reachable: false,
                container_running: true,
                error_message: Some("Service is not responding to HTTP requests".to_string()),
                logs_tail: logs,
            }
        }
    }

    /// 等待容器启动并进行健康检查
    async fn wait_for_container_ready(
        &self,
        container_id: &str,
        port: u16,
        max_wait_secs: u64,
    ) -> HealthCheckResult {
        let start = std::time::Instant::now();
        let max_duration = Duration::from_secs(max_wait_secs);

        info!(
            "Waiting for container {} to be ready (timeout: {}s)...",
            container_id, max_wait_secs
        );

        while start.elapsed() < max_duration {
            // 先检查容器是否还在运行
            if !self.is_container_running(container_id).await {
                error!(
                    "Container {} stopped unexpectedly after starting. This usually indicates a configuration error or missing dependencies.",
                    container_id
                );
                let logs = self.get_container_logs(container_id, Some(100)).await.ok();

                // 尝试从日志中提取错误信息
                let error_hint = logs
                    .as_ref()
                    .and_then(|l| extract_error_hint(l))
                    .unwrap_or_default();

                let error_message = if error_hint.is_empty() {
                    "Container stopped unexpectedly after starting. Please check the logs for details.".to_string()
                } else {
                    format!(
                        "Container stopped unexpectedly. Possible cause: {}",
                        error_hint
                    )
                };

                return HealthCheckResult {
                    status: HealthStatus::Unhealthy,
                    http_reachable: false,
                    container_running: false,
                    error_message: Some(error_message),
                    logs_tail: logs,
                };
            }

            // 检查 HTTP 是否可达
            if self.check_http_health(port).await {
                info!("Container {} is ready and responding to HTTP requests", container_id);
                return HealthCheckResult {
                    status: HealthStatus::Healthy,
                    http_reachable: true,
                    container_running: true,
                    error_message: None,
                    logs_tail: None,
                };
            }

            debug!(
                "Container {} not ready yet, retrying... (elapsed: {:.1}s)",
                container_id,
                start.elapsed().as_secs_f32()
            );
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        // 超时，获取日志
        let is_running = self.is_container_running(container_id).await;
        warn!(
            "Container {} health check timed out after {}s (container running: {})",
            container_id, max_wait_secs, is_running
        );

        let logs = self.get_container_logs(container_id, Some(100)).await.ok();

        let error_message = if is_running {
            format!(
                "Service started but did not respond to HTTP requests within {} seconds. The container is still running - check if the service is binding to the correct port (8080).",
                max_wait_secs
            )
        } else {
            "Service failed to start within the timeout period. Check the logs for startup errors.".to_string()
        };

        HealthCheckResult {
            status: HealthStatus::Unhealthy,
            http_reachable: false,
            container_running: is_running,
            error_message: Some(error_message),
            logs_tail: logs,
        }
    }

    /// 检查镜像是否存在
    async fn image_exists(&self, image: &str) -> bool {
        self.docker.inspect_image(image).await.is_ok()
    }

    /// 拉取 Docker 镜像
    ///
    /// 返回 Ok(true) 表示拉取成功，Ok(false) 表示镜像已存在无需拉取
    pub async fn pull_image(&self, image: &str) -> Result<bool> {
        // 检查镜像是否已存在
        if self.image_exists(image).await {
            info!("镜像已存在，无需拉取: {}", image);
            return Ok(false);
        }

        info!("开始拉取镜像: {}", image);

        // 解析镜像名称和标签
        let (image_name, tag) = if let Some(pos) = image.rfind(':') {
            (&image[..pos], &image[pos + 1..])
        } else {
            (image, "latest")
        };

        let options = CreateImageOptions {
            from_image: Some(image_name.to_string()),
            tag: Some(tag.to_string()),
            ..Default::default()
        };

        // 拉取镜像（流式处理进度）
        let mut stream = self.docker.create_image(Some(options), None, None);

        while let Some(result) = stream.try_next().await? {
            // 记录拉取进度
            if let Some(status) = result.status {
                if let Some(progress) = result.progress {
                    debug!("[Pull] {}: {}", status, progress);
                } else {
                    debug!("[Pull] {}", status);
                }
            }
        }

        // 验证镜像是否拉取成功
        if self.image_exists(image).await {
            info!("镜像拉取成功: {}", image);
            Ok(true)
        } else {
            anyhow::bail!("镜像拉取后仍不存在: {}", image)
        }
    }

    /// 部署新的 EchoKit 容器
    pub async fn deploy(
        &self,
        echokit_config: EchoKitConfig,
        port: Option<u16>,
        user_id: Option<&str>,
    ) -> Result<DeployResponse> {
        let container_name = echokit_config.name.clone();
        let port = match port {
            Some(p) => p,
            None => self.allocate_port().await.context("Failed to allocate port")?,
        };

        info!(
            "[1/6] 准备部署: 容器名='{}', 端口={}, 镜像='{}'",
            container_name, port, self.config.docker_image
        );

        // 生成配置文件
        info!("[2/6] 生成配置文件...");
        let config_content = generate_config_toml(&echokit_config);
        let config_dir = Path::new(&self.config.config_dir).join(&container_name);

        debug!("创建配置目录: {:?}", config_dir);
        fs::create_dir_all(&config_dir)
            .await
            .context(format!("Failed to create config directory: {:?}", config_dir))?;

        let config_path = config_dir.join("config.toml");
        debug!("写入配置文件: {:?}", config_path);
        debug!("生成的 config.toml 内容:\n{}", config_content);

        fs::write(&config_path, &config_content)
            .await
            .context(format!("Failed to write config file: {:?}", config_path))?;

        // 复制 hello.wav
        let hello_wav_dest = config_dir.join("hello.wav");
        if Path::new(&self.config.hello_wav_path).exists() {
            debug!("复制 hello.wav: {:?}", self.config.hello_wav_path);
            fs::copy(&self.config.hello_wav_path, &hello_wav_dest)
                .await
                .context("Failed to copy hello.wav")?;
        } else {
            debug!("hello.wav 不存在，跳过: {:?}", self.config.hello_wav_path);
        }

        // 创建录音目录
        let record_dir = Path::new(&self.config.record_dir).join(&container_name);
        debug!("创建录音目录: {:?}", record_dir);
        fs::create_dir_all(&record_dir)
            .await
            .context(format!("Failed to create record directory: {:?}", record_dir))?;

        info!("[2/6] 配置文件生成完成: {:?}", config_path);

        // 检查并拉取镜像
        info!("[3/6] 检查 Docker 镜像...");
        if !self.image_exists(&self.config.docker_image).await {
            info!(
                "[3/6] 镜像不存在，开始拉取: {}",
                self.config.docker_image
            );
            self.pull_image(&self.config.docker_image)
                .await
                .context(format!(
                    "Failed to pull image '{}'. Please check your network connection and ensure the image exists on Docker Hub.",
                    self.config.docker_image
                ))?;
            info!("[3/6] 镜像拉取完成");
        } else {
            info!("[3/6] 镜像已存在: {}", self.config.docker_image);
        }

        // 配置端口映射
        let mut port_bindings = HashMap::new();
        port_bindings.insert(
            "8080/tcp".to_string(),
            Some(vec![PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(port.to_string()),
            }]),
        );

        // 配置卷挂载
        let config_path_abs = fs::canonicalize(&config_path)
            .await
            .context(format!("Failed to resolve config path: {:?}", config_path))?;
        let record_dir_abs = fs::canonicalize(&record_dir)
            .await
            .context(format!("Failed to resolve record directory: {:?}", record_dir))?;

        let mut binds = vec![
            format!("{}:/app/config.toml:ro", config_path_abs.display()),
            format!("{}:/app/record", record_dir_abs.display()),
        ];

        if hello_wav_dest.exists() {
            let hello_wav_abs = fs::canonicalize(&hello_wav_dest)
                .await
                .context("Failed to resolve hello.wav path")?;
            binds.push(format!("{}:/app/hello.wav:ro", hello_wav_abs.display()));
        }

        debug!("Volume bindings: {:?}", binds);

        let host_config = HostConfig {
            port_bindings: Some(port_bindings),
            binds: Some(binds),
            ..Default::default()
        };

        // 创建容器配置
        let env = vec![
            "RUST_LOG=info".to_string(),
            format!("CONTAINER_NAME={}", container_name),
        ];

        // 添加标签以标识 EchoKit 管理的容器
        let mut labels = HashMap::new();
        labels.insert("managed-by".to_string(), "echokit-console".to_string());

        let container_config = ContainerCreateBody {
            image: Some(self.config.docker_image.clone()),
            env: Some(env),
            host_config: Some(host_config),
            labels: Some(labels),
            ..Default::default()
        };

        // 创建容器
        let options = CreateContainerOptions {
            name: Some(container_name.clone()),
            ..Default::default()
        };

        info!(
            "[4/6] 创建 Docker 容器: 镜像='{}', 端口映射={}:8080",
            self.config.docker_image, port
        );

        let response = self
            .docker
            .create_container(Some(options), container_config)
            .await
            .context(format!(
                "Failed to create container '{}'. Docker daemon may not be running or there was an unexpected error.",
                container_name
            ))?;

        info!(
            "[4/6] 容器创建成功: id={}",
            &response.id[..12.min(response.id.len())]
        );

        // 启动容器
        info!("[5/6] 启动容器...");
        self.docker
            .start_container(&response.id, None::<StartContainerOptions>)
            .await
            .context(format!(
                "Failed to start container '{}'. The container was created but failed to start. Check Docker logs for details.",
                container_name
            ))?;

        info!("[5/6] 容器启动成功");

        // 等待容器就绪并进行健康检查
        info!("[6/6] 等待服务就绪，执行健康检查...");
        let health = self.wait_for_container_ready(&response.id, port, 30).await;

        if health.status == HealthStatus::Healthy {
            info!("[6/6] 健康检查通过，服务已就绪");
        } else {
            warn!(
                "[6/6] 健康检查未通过: {:?}",
                health.error_message.as_deref().unwrap_or("未知原因")
            );
        }

        let status = if health.status == HealthStatus::Healthy {
            ContainerStatus::Running
        } else if health.container_running {
            ContainerStatus::Error
        } else {
            ContainerStatus::Stopped
        };

        let container_host = self.config.get_container_host();
        let ws_url = format!("ws://{}:{}/ws/{{device_id}}", container_host, port);

        // 将容器信息写入数据库
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        sqlx::query(
            r#"
            INSERT INTO containers (id, name, host, port, use_tls, is_default, is_external, created_at, user_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                host = EXCLUDED.host,
                port = EXCLUDED.port,
                use_tls = EXCLUDED.use_tls,
                updated_at = $8
            "#,
        )
        .bind(&response.id)
        .bind(&container_name)
        .bind(&container_host)
        .bind(port as i32)
        .bind(false) // use_tls
        .bind(false) // is_default
        .bind(false) // is_external
        .bind(now)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .context("Failed to insert container info to database")?;

        info!("容器信息已写入数据库: id={}, name={}, port={}, user_id={:?}", response.id, container_name, port, user_id);

        Ok(DeployResponse {
            container_id: response.id,
            container_name,
            port,
            ws_url,
            status,
            health,
        })
    }

    /// 获取用户可见的 EchoKit 容器（用户自己的 + 全局共享的）
    pub async fn list_containers_for_user(&self, user_id: &str) -> Result<Vec<ContainerInfo>> {
        // 从数据库获取用户可见的容器 ID 列表（用户自己的 + 全局共享的）
        let visible_container_ids: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT id FROM containers
            WHERE user_id = $1 OR user_id IS NULL
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to query visible containers")?;

        // 获取所有容器，然后过滤
        let all_containers = self.list_containers().await?;
        let filtered: Vec<ContainerInfo> = all_containers
            .into_iter()
            .filter(|c| visible_container_ids.contains(&c.id))
            .collect();

        Ok(filtered)
    }

    /// 获取所有 EchoKit 容器（内部使用，不做用户过滤）
    pub async fn list_containers(&self) -> Result<Vec<ContainerInfo>> {
        let mut filters = HashMap::new();
        filters.insert("label".to_string(), vec!["managed-by=echokit-console".to_string()]);

        let options = ListContainersOptions {
            all: true,
            filters: Some(filters),
            ..Default::default()
        };

        let containers = self.docker.list_containers(Some(options)).await?;
        let mut result = Vec::new();

        for container in containers {
            let id = container.id.unwrap_or_default();
            let name = container
                .names
                .and_then(|n| n.first().cloned())
                .unwrap_or_default()
                .trim_start_matches('/')
                .to_string();

            let port = container
                .ports
                .and_then(|ports| {
                    ports
                        .iter()
                        .find_map(|p| p.public_port.map(|pp| pp as u16))
                })
                .unwrap_or(0);

            let status = match container.state {
                Some(ContainerSummaryStateEnum::RUNNING) => ContainerStatus::Running,
                Some(ContainerSummaryStateEnum::EXITED) => ContainerStatus::Stopped,
                Some(ContainerSummaryStateEnum::CREATED) => ContainerStatus::Creating,
                _ => ContainerStatus::Error,
            };

            let created_at = container
                .created
                .map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_default())
                .unwrap_or_else(Utc::now);

            let container_host = self.config.get_container_host();
            let ws_url = format!("ws://{}:{}/ws/{{device_id}}", container_host, port);

            result.push(ContainerInfo {
                id,
                name,
                port,
                ws_url,
                status,
                created_at,
                health: None, // 列表查询不做健康检查，可通过单独接口获取
            });
        }

        Ok(result)
    }

    /// 验证用户是否有权限操作指定容器
    ///
    /// 规则：
    /// - 容器 user_id 为 NULL：全局共享容器，只读（不能删除/停止/启动）
    /// - 容器 user_id 与当前用户相同：用户自己的容器，可操作
    /// - 其他情况：无权限
    pub async fn check_container_permission(&self, container_id: &str, user_id: &str, allow_shared: bool) -> Result<bool> {
        let row: Option<(Option<String>,)> = sqlx::query_as(
            r#"SELECT user_id FROM containers WHERE id = $1"#,
        )
        .bind(container_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query container")?;

        match row {
            None => Ok(false), // 容器不存在
            Some((None,)) => Ok(allow_shared), // 全局共享容器
            Some((Some(owner_id),)) => Ok(owner_id == user_id), // 检查所有权
        }
    }

    /// 获取单个容器信息（包含健康检查）- 带用户权限验证
    pub async fn get_container_for_user(&self, id: &str, user_id: &str) -> Result<ContainerInfo> {
        // 验证用户有权限查看（包括共享容器）
        if !self.check_container_permission(id, user_id, true).await? {
            anyhow::bail!("Container not found or access denied");
        }
        self.get_container(id).await
    }

    /// 获取单个容器信息（包含健康检查）
    pub async fn get_container(&self, id: &str) -> Result<ContainerInfo> {
        let containers = self.list_containers().await?;
        let mut container = containers
            .into_iter()
            .find(|c| c.id == id || c.name == id)
            .context("Container not found")?;

        // 对单个容器查询执行健康检查
        if container.status == ContainerStatus::Running && container.port > 0 {
            let health = self.health_check(&container.id, container.port).await;
            container.health = Some(health);
        }

        Ok(container)
    }

    /// 停止容器 - 带用户权限验证
    pub async fn stop_container_for_user(&self, id: &str, user_id: &str) -> Result<()> {
        // 只有容器所有者可以停止（不允许操作共享容器）
        if !self.check_container_permission(id, user_id, false).await? {
            anyhow::bail!("Container not found or access denied");
        }
        self.stop_container(id).await
    }

    /// 停止容器
    pub async fn stop_container(&self, id: &str) -> Result<()> {
        let options = StopContainerOptions {
            t: Some(10),
            ..Default::default()
        };
        self.docker
            .stop_container(id, Some(options))
            .await
            .context("Failed to stop container")?;
        Ok(())
    }

    /// 启动容器 - 带用户权限验证
    pub async fn start_container_for_user(&self, id: &str, user_id: &str) -> Result<()> {
        // 只有容器所有者可以启动（不允许操作共享容器）
        if !self.check_container_permission(id, user_id, false).await? {
            anyhow::bail!("Container not found or access denied");
        }
        self.start_container(id).await
    }

    /// 启动容器
    pub async fn start_container(&self, id: &str) -> Result<()> {
        self.docker
            .start_container(id, None::<StartContainerOptions>)
            .await
            .context("Failed to start container")?;
        Ok(())
    }

    /// 删除容器 - 带用户权限验证
    pub async fn delete_container_for_user(&self, id: &str, user_id: &str) -> Result<()> {
        // 只有容器所有者可以删除（不允许删除共享容器）
        if !self.check_container_permission(id, user_id, false).await? {
            anyhow::bail!("Container not found or access denied");
        }
        self.delete_container(id).await
    }

    /// 删除容器
    pub async fn delete_container(&self, id: &str) -> Result<()> {
        // 先尝试停止
        let _ = self.stop_container(id).await;

        let options = RemoveContainerOptions {
            force: true,
            ..Default::default()
        };
        self.docker
            .remove_container(id, Some(options))
            .await
            .context("Failed to remove container")?;

        // 从数据库删除容器记录
        sqlx::query("DELETE FROM containers WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete container from database")?;

        Ok(())
    }

    /// 获取容器日志 - 带用户权限验证
    pub async fn get_container_logs_for_user(&self, id: &str, user_id: &str, tail: Option<usize>) -> Result<String> {
        // 用户可以查看自己的容器和共享容器的日志
        if !self.check_container_permission(id, user_id, true).await? {
            anyhow::bail!("Container not found or access denied");
        }
        self.get_container_logs(id, tail).await
    }

    /// 获取容器日志
    pub async fn get_container_logs(&self, id: &str, tail: Option<usize>) -> Result<String> {
        use futures_util::StreamExt;

        let options = LogsOptions {
            stdout: true,
            stderr: true,
            tail: tail
                .map(|t| t.to_string())
                .unwrap_or_else(|| "100".to_string()),
            ..Default::default()
        };

        let mut logs = self.docker.logs(id, Some(options));
        let mut output = String::new();

        while let Some(log) = logs.next().await {
            match log {
                Ok(chunk) => output.push_str(&chunk.to_string()),
                Err(e) => return Err(e.into()),
            }
        }

        Ok(output)
    }
}
