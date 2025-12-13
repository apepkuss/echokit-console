#!/bin/bash

# EchoKit Console 开发环境停止脚本

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

log_info "停止 EchoKit Console 开发环境..."
echo ""

# ============================================================
# 停止 Frontend
# ============================================================
log_info "停止 Frontend 服务..."
if [ -f "logs/frontend.pid" ]; then
    FRONTEND_PID=$(cat logs/frontend.pid)
    if ps -p $FRONTEND_PID > /dev/null 2>&1; then
        kill $FRONTEND_PID
        log_success "Frontend 已停止 (PID: $FRONTEND_PID)"
    else
        log_warn "Frontend 进程不存在"
    fi
    rm -f logs/frontend.pid
else
    # 尝试通过端口查找进程
    if lsof -ti :5173 > /dev/null 2>&1; then
        PIDS=$(lsof -ti :5173)
        kill $PIDS
        log_success "已停止占用端口 5173 的进程"
    else
        log_warn "未找到 Frontend 进程"
    fi
fi

# ============================================================
# 停止 Proxy
# ============================================================
log_info "停止 Proxy 服务..."
if [ -f "logs/proxy.pid" ]; then
    PROXY_PID=$(cat logs/proxy.pid)
    if ps -p $PROXY_PID > /dev/null 2>&1; then
        kill $PROXY_PID
        log_success "Proxy 已停止 (PID: $PROXY_PID)"
    else
        log_warn "Proxy 进程不存在"
    fi
    rm -f logs/proxy.pid
else
    # 尝试通过端口查找进程
    if lsof -ti :10086 > /dev/null 2>&1; then
        PIDS=$(lsof -ti :10086)
        kill $PIDS
        log_success "已停止占用端口 10086 的进程"
    else
        log_warn "未找到 Proxy 进程"
    fi
fi

# ============================================================
# 停止 Backend
# ============================================================
log_info "停止 Backend 服务..."
if [ -f "logs/backend.pid" ]; then
    BACKEND_PID=$(cat logs/backend.pid)
    if ps -p $BACKEND_PID > /dev/null 2>&1; then
        kill $BACKEND_PID
        log_success "Backend 已停止 (PID: $BACKEND_PID)"
    else
        log_warn "Backend 进程不存在"
    fi
    rm -f logs/backend.pid
else
    # 尝试通过端口查找进程
    if lsof -ti :3000 > /dev/null 2>&1; then
        PIDS=$(lsof -ti :3000)
        kill $PIDS
        log_success "已停止占用端口 3000 的进程"
    else
        log_warn "未找到 Backend 进程"
    fi
fi

# ============================================================
# 可选：停止数据服务容器
# ============================================================
echo ""
read -p "是否停止 PostgreSQL 和 Redis 容器？(y/N): " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    log_info "停止 PostgreSQL 容器..."
    if docker ps --format '{{.Names}}' | grep -q "^echokit-postgres$"; then
        docker stop echokit-postgres
        log_success "PostgreSQL 容器已停止"
    else
        log_warn "PostgreSQL 容器未运行"
    fi

    log_info "停止 Redis 容器..."
    if docker ps --format '{{.Names}}' | grep -q "^echokit-redis$"; then
        docker stop echokit-redis
        log_success "Redis 容器已停止"
    else
        log_warn "Redis 容器未运行"
    fi
else
    log_info "保持 PostgreSQL 和 Redis 容器运行"
fi

echo ""
log_success "EchoKit Console 开发环境已停止"
