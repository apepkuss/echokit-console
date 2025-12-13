#!/bin/bash

# EchoKit Console 开发环境一键启动脚本
# 用途：本地开发模式 - PostgreSQL 容器化，其他服务本地运行

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

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

log_info "EchoKit Console 开发环境启动中..."
echo ""

# ============================================================
# 1. 启动 PostgreSQL 容器
# ============================================================
log_info "步骤 1/5: 启动 PostgreSQL 数据库..."

if docker ps -a --format '{{.Names}}' | grep -q "^echokit-postgres$"; then
    if docker ps --format '{{.Names}}' | grep -q "^echokit-postgres$"; then
        log_success "PostgreSQL 容器已在运行"
    else
        log_info "启动已存在的 PostgreSQL 容器..."
        docker start echokit-postgres
        log_success "PostgreSQL 容器已启动"
    fi
else
    log_info "创建并启动新的 PostgreSQL 容器..."
    docker run -d \
        --name echokit-postgres \
        -e POSTGRES_DB=echokit \
        -e POSTGRES_USER=echokit \
        -e POSTGRES_PASSWORD=echokit \
        -p 5432:5432 \
        -v echokit_postgres_data:/var/lib/postgresql/data \
        postgres:16-alpine
    log_success "PostgreSQL 容器创建成功"
fi

# 等待数据库启动
log_info "等待数据库就绪..."
sleep 3

# 验证数据库连接
if docker exec echokit-postgres pg_isready -U echokit > /dev/null 2>&1; then
    log_success "数据库连接正常"
else
    log_error "数据库连接失败"
    exit 1
fi

echo ""

# ============================================================
# 2. 启动 Redis 容器
# ============================================================
log_info "步骤 2/5: 启动 Redis 服务..."

if docker ps -a --format '{{.Names}}' | grep -q "^echokit-redis$"; then
    if docker ps --format '{{.Names}}' | grep -q "^echokit-redis$"; then
        log_success "Redis 容器已在运行"
    else
        log_info "启动已存在的 Redis 容器..."
        docker start echokit-redis
        log_success "Redis 容器已启动"
    fi
else
    log_info "创建并启动新的 Redis 容器..."
    docker run -d \
        --name echokit-redis \
        -p 6379:6379 \
        redis:7-alpine
    log_success "Redis 容器创建成功"
fi

# 等待 Redis 启动
log_info "等待 Redis 就绪..."
sleep 2

# 验证 Redis 连接
if docker exec echokit-redis redis-cli ping > /dev/null 2>&1; then
    log_success "Redis 连接正常"
else
    log_error "Redis 连接失败"
    exit 1
fi

echo ""

# ============================================================
# 3. 启动 Backend
# ============================================================
log_info "步骤 3/5: 启动 Backend 服务..."

# 检查 Backend 目录
if [ ! -d "backend" ]; then
    log_error "Backend 目录不存在"
    exit 1
fi

# 检查是否已有 Backend 进程在运行
if lsof -i :3000 > /dev/null 2>&1; then
    log_warn "端口 3000 已被占用，跳过 Backend 启动"
else
    log_info "启动 Backend (端口 3000)..."
    cd backend

    # 后台运行 Backend
    RUST_LOG=info cargo run -r > ../logs/backend.log 2>&1 &
    BACKEND_PID=$!
    echo $BACKEND_PID > ../logs/backend.pid

    cd ..
    log_success "Backend 已启动 (PID: $BACKEND_PID)"
fi

echo ""

# ============================================================
# 4. 启动 Proxy
# ============================================================
log_info "步骤 4/5: 启动 Proxy 服务..."

# 检查 Proxy 目录
if [ ! -d "proxy" ]; then
    log_error "Proxy 目录不存在"
    exit 1
fi

# 检查是否已有 Proxy 进程在运行
if lsof -i :10086 > /dev/null 2>&1; then
    log_warn "端口 10086 已被占用，跳过 Proxy 启动"
else
    log_info "启动 Proxy (端口 10086, 10087)..."
    cd proxy

    # 后台运行 Proxy
    RUST_LOG=info cargo run -r > ../logs/proxy.log 2>&1 &
    PROXY_PID=$!
    echo $PROXY_PID > ../logs/proxy.pid

    cd ..
    log_success "Proxy 已启动 (PID: $PROXY_PID)"
fi

echo ""

# ============================================================
# 5. 启动 Frontend
# ============================================================
log_info "步骤 5/5: 启动 Frontend 服务..."

# 检查 Frontend 目录
if [ ! -d "frontend" ]; then
    log_error "Frontend 目录不存在"
    exit 1
fi

# 检查是否已有 Frontend 进程在运行
if lsof -i :5173 > /dev/null 2>&1; then
    log_warn "端口 5173 已被占用，跳过 Frontend 启动"
else
    log_info "启动 Frontend (端口 5173)..."
    cd frontend

    # 检查 node_modules
    if [ ! -d "node_modules" ]; then
        log_info "安装 Frontend 依赖..."
        npm install
    fi

    # 后台运行 Frontend
    npm run dev > ../logs/frontend.log 2>&1 &
    FRONTEND_PID=$!
    echo $FRONTEND_PID > ../logs/frontend.pid

    cd ..
    log_success "Frontend 已启动 (PID: $FRONTEND_PID)"
fi

echo ""

# ============================================================
# 等待服务就绪
# ============================================================
log_info "等待服务编译和启动（Rust 首次编译需要约 15-20 秒）..."
sleep 20

# ============================================================
# 验证服务状态
# ============================================================
echo ""
log_info "服务状态检查:"
echo ""

# 检查 PostgreSQL
if docker ps --format '{{.Names}}' | grep -q "^echokit-postgres$"; then
    echo -e "  ${GREEN}✓${NC} PostgreSQL  : localhost:5432"
else
    echo -e "  ${RED}✗${NC} PostgreSQL  : 未运行"
fi

# 检查 Redis
if docker ps --format '{{.Names}}' | grep -q "^echokit-redis$"; then
    echo -e "  ${GREEN}✓${NC} Redis       : localhost:6379"
else
    echo -e "  ${RED}✗${NC} Redis       : 未运行"
fi

# 检查 Backend
if curl -s http://localhost:3000/api/health > /dev/null 2>&1; then
    echo -e "  ${GREEN}✓${NC} Backend     : http://localhost:3000"
else
    echo -e "  ${YELLOW}⚠${NC} Backend     : 启动中或未响应"
fi

# 检查 Proxy
if curl -s http://localhost:10087/health > /dev/null 2>&1; then
    echo -e "  ${GREEN}✓${NC} Proxy       : ws://localhost:10086 (health: http://localhost:10087)"
else
    echo -e "  ${YELLOW}⚠${NC} Proxy       : 启动中或未响应"
fi

# 检查 Frontend
if curl -s http://localhost:5173 > /dev/null 2>&1; then
    echo -e "  ${GREEN}✓${NC} Frontend    : http://localhost:5173"
else
    echo -e "  ${YELLOW}⚠${NC} Frontend    : 启动中或未响应"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
log_success "EchoKit Console 开发环境启动完成！"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "访问地址:"
echo "  - Frontend:  http://localhost:5173"
echo "  - Backend:   http://localhost:3000"
echo "  - Proxy:     ws://localhost:10086"
echo ""
echo "日志位置:"
echo "  - Backend:   logs/backend.log"
echo "  - Proxy:     logs/proxy.log"
echo "  - Frontend:  logs/frontend.log"
echo ""
echo "停止服务:"
echo "  ./stop-dev.sh"
echo ""
