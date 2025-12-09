# 构建阶段
FROM rust:1.83-slim as builder

# 安装编译依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 创建虚拟 main.rs 用于缓存依赖
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制源代码
COPY src ./src

# 构建应用
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN useradd -m -u 1000 echokit

# 设置工作目录
WORKDIR /app

# 创建数据目录
RUN mkdir -p /app/data/configs /app/data/records && \
    chown -R echokit:echokit /app/data

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/echokit-console /usr/local/bin/echokit-backend

# 复制静态文件（如果有）
COPY data /app/data

# 切换到非 root 用户
USER echokit

# 暴露端口
EXPOSE 3000

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/health || exit 1

# 启动应用
CMD ["echokit-backend"]
