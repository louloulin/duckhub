#!/bin/bash

# DuckHub 金融数据平台启动脚本
# 自动启动前后端服务

set -e

# 颜色定义
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

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log_info "检查系统依赖..."
    
    # 检查 Rust
    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo 未安装，请先安装 Rust: https://rustup.rs/"
        exit 1
    fi
    
    # 检查 Node.js
    if ! command -v node &> /dev/null; then
        log_error "Node.js 未安装，请先安装 Node.js 18+: https://nodejs.org/"
        exit 1
    fi
    
    # 检查 npm
    if ! command -v npm &> /dev/null; then
        log_error "npm 未安装，请先安装 npm"
        exit 1
    fi
    
    log_success "依赖检查完成"
}

# 编译后端服务
build_backend() {
    log_info "编译后端服务..."
    
    # 编译 web-api 服务
    log_info "编译 web-api 服务..."
    cargo build --release --bin duckhub-web-api
    
    if [ $? -eq 0 ]; then
        log_success "后端编译完成"
    else
        log_error "后端编译失败"
        exit 1
    fi
}

# 安装前端依赖
install_frontend_deps() {
    log_info "安装前端依赖..."
    
    cd crates/web-frontend
    
    if [ ! -d "node_modules" ]; then
        log_info "首次安装前端依赖..."
        npm install
    else
        log_info "前端依赖已存在，跳过安装"
    fi
    
    cd ../..
    log_success "前端依赖安装完成"
}

# 启动后端服务
start_backend() {
    log_info "启动后端 API 服务..."
    
    # 设置环境变量
    export RUST_LOG=info
    export DUCKHUB_HOST=0.0.0.0
    export DUCKHUB_PORT=8080
    
    # 启动 web-api 服务
    cargo run --release --bin duckhub-web-api &
    BACKEND_PID=$!
    
    log_info "后端服务启动中... PID: $BACKEND_PID"
    
    # 等待服务启动
    log_info "等待后端服务启动..."
    sleep 5
    
    # 检查服务是否启动成功
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        log_success "后端服务启动成功 - http://localhost:8080"
    else
        log_warning "后端服务可能还在启动中，请稍后检查"
    fi
}

# 启动前端服务
start_frontend() {
    log_info "启动前端开发服务器..."
    
    cd crates/web-frontend
    
    # 设置环境变量
    export VITE_API_URL=http://localhost:8080
    
    # 启动前端开发服务器
    npm run dev &
    FRONTEND_PID=$!
    
    log_info "前端服务启动中... PID: $FRONTEND_PID"
    
    cd ../..
    
    # 等待前端服务启动
    log_info "等待前端服务启动..."
    sleep 3
    
    log_success "前端服务启动成功 - http://localhost:3000"
}

# 显示服务状态
show_status() {
    echo ""
    log_info "=== DuckHub 服务状态 ==="
    echo ""
    
    # 检查后端服务
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        log_success "✅ 后端 API 服务: http://localhost:8080"
        log_info "   - 健康检查: http://localhost:8080/health"
        log_info "   - API 文档: http://localhost:8080/docs"
        log_info "   - 指标监控: http://localhost:8080/metrics"
    else
        log_warning "❌ 后端 API 服务: 未响应"
    fi
    
    # 检查前端服务
    if curl -s http://localhost:3000 > /dev/null 2>&1; then
        log_success "✅ 前端 Web 应用: http://localhost:3000"
    else
        log_warning "❌ 前端 Web 应用: 未响应"
    fi
    
    echo ""
    log_info "=== 功能模块 ==="
    echo ""
    log_info "🏠 仪表板: http://localhost:3000/"
    log_info "📊 数据查询: http://localhost:3000/query"
    log_info "🗃️ DuckLake 管理: http://localhost:3000/ducklake"
    log_info "🤖 AI 助手: http://localhost:3000/ai"
    log_info "⚙️ 系统设置: http://localhost:3000/settings"
    echo ""
}

# 清理函数
cleanup() {
    log_info "正在关闭服务..."
    
    if [ ! -z "$BACKEND_PID" ]; then
        kill $BACKEND_PID 2>/dev/null || true
        log_info "后端服务已关闭"
    fi
    
    if [ ! -z "$FRONTEND_PID" ]; then
        kill $FRONTEND_PID 2>/dev/null || true
        log_info "前端服务已关闭"
    fi
    
    # 清理可能的僵尸进程
    pkill -f "duckhub-web-api" 2>/dev/null || true
    pkill -f "vite" 2>/dev/null || true
    
    log_success "服务清理完成"
    exit 0
}

# 设置信号处理
trap cleanup SIGINT SIGTERM

# 主函数
main() {
    echo ""
    log_info "🚀 启动 DuckHub 金融数据平台"
    echo ""
    
    # 检查依赖
    check_dependencies
    
    # 编译后端
    build_backend
    
    # 安装前端依赖
    install_frontend_deps
    
    # 启动后端服务
    start_backend
    
    # 启动前端服务
    start_frontend
    
    # 显示状态
    show_status
    
    log_info "🎉 DuckHub 启动完成！"
    log_info "按 Ctrl+C 停止所有服务"
    
    # 保持脚本运行
    while true; do
        sleep 1
    done
}

# 帮助信息
show_help() {
    echo "DuckHub 金融数据平台启动脚本"
    echo ""
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  -h, --help     显示帮助信息"
    echo "  -b, --backend  仅启动后端服务"
    echo "  -f, --frontend 仅启动前端服务"
    echo "  -c, --check    检查服务状态"
    echo "  -s, --stop     停止所有服务"
    echo ""
    echo "示例:"
    echo "  $0              # 启动完整服务"
    echo "  $0 --backend    # 仅启动后端"
    echo "  $0 --frontend   # 仅启动前端"
    echo "  $0 --check      # 检查服务状态"
    echo ""
}

# 仅启动后端
start_backend_only() {
    log_info "🚀 启动后端服务"
    check_dependencies
    build_backend
    start_backend
    
    log_info "后端服务已启动，按 Ctrl+C 停止"
    while true; do
        sleep 1
    done
}

# 仅启动前端
start_frontend_only() {
    log_info "🚀 启动前端服务"
    check_dependencies
    install_frontend_deps
    start_frontend
    
    log_info "前端服务已启动，按 Ctrl+C 停止"
    while true; do
        sleep 1
    done
}

# 检查服务状态
check_status() {
    log_info "检查 DuckHub 服务状态..."
    show_status
}

# 停止所有服务
stop_services() {
    log_info "停止 DuckHub 服务..."
    
    # 停止可能运行的服务
    pkill -f "duckhub-web-api" 2>/dev/null || true
    pkill -f "vite" 2>/dev/null || true
    pkill -f "node.*vite" 2>/dev/null || true
    
    log_success "所有服务已停止"
}

# 解析命令行参数
case "${1:-}" in
    -h|--help)
        show_help
        exit 0
        ;;
    -b|--backend)
        start_backend_only
        ;;
    -f|--frontend)
        start_frontend_only
        ;;
    -c|--check)
        check_status
        exit 0
        ;;
    -s|--stop)
        stop_services
        exit 0
        ;;
    "")
        main
        ;;
    *)
        log_error "未知选项: $1"
        show_help
        exit 1
        ;;
esac
