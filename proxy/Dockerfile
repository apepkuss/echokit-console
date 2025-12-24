# 构建阶段
FROM rust:1.92-slim as builder

# 安装编译依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 复制 workspace 根目录的 Cargo.toml 和 Cargo.lock
COPY Cargo.toml Cargo.lock ./

# 复制子项目的 Cargo.toml（用于依赖缓存）
COPY backend/Cargo.toml ./backend/
COPY proxy/Cargo.toml ./proxy/

# 创建虚拟 main.rs 用于缓存依赖
RUN mkdir -p backend/src proxy/src && \
    echo "fn main() {}" > backend/src/main.rs && \
    echo "fn main() {}" > proxy/src/main.rs && \
    cargo build --release -p echokit-proxy && \
    rm -rf proxy/src target/release/echokit-proxy target/release/deps/echokit_proxy*

# 复制 proxy 源代码（保留 backend 的虚拟代码以满足 workspace 要求）
COPY proxy/src ./proxy/src

# 构建 proxy 应用
RUN cargo build --release -p echokit-proxy

# 运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN useradd -m -u 1000 echokit

# 设置工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/echokit-proxy /usr/local/bin/echokit-proxy

# 切换到非 root 用户
USER echokit

# 暴露端口
EXPOSE 10086 10087

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:10087/health || exit 1

# 启动应用
CMD ["echokit-proxy"]
