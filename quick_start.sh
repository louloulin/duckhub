#!/bin/bash

# DuckHub 快速启动脚本 - 简化版本
# 启动前端和简化的后端服务

set -e

echo "🚀 DuckHub 快速启动"
echo "=================="

# 检查依赖
check_deps() {
    if ! command -v cargo &> /dev/null; then
        echo "❌ 请先安装 Rust: https://rustup.rs/"
        exit 1
    fi
    
    if ! command -v node &> /dev/null; then
        echo "❌ 请先安装 Node.js: https://nodejs.org/"
        exit 1
    fi
    
    echo "✅ 依赖检查通过"
}

# 启动简化后端
start_simple_backend() {
    echo "📡 启动简化后端服务..."
    
    # 编译并运行简化后端
    cargo run --bin simple_backend &
    BACKEND_PID=$!
    
    echo "后端服务 PID: $BACKEND_PID"
    echo "等待后端服务启动..."
    sleep 5
    
    # 检查服务
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        echo "✅ 后端服务启动成功: http://localhost:8080"
    else
        echo "⚠️  后端服务启动中..."
    fi
}

# 启动前端（如果还没启动）
start_frontend() {
    # 检查前端是否已经在运行
    if curl -s http://localhost:3000 > /dev/null 2>&1; then
        echo "✅ 前端服务已在运行: http://localhost:3000"
        return
    fi
    
    echo "🌐 启动前端服务..."
    
    cd crates/web-frontend
    
    # 安装依赖（如果需要）
    if [ ! -d "node_modules" ]; then
        echo "📦 安装前端依赖..."
        npm install
    fi
    
    # 启动前端
    npm run dev &
    FRONTEND_PID=$!
    
    cd ../..
    
    echo "前端服务 PID: $FRONTEND_PID"
    echo "✅ 前端服务启动: http://localhost:3000"
}

# 清理函数
cleanup() {
    echo ""
    echo "🛑 关闭服务..."
    
    if [ ! -z "$BACKEND_PID" ]; then
        kill $BACKEND_PID 2>/dev/null || true
    fi
    
    if [ ! -z "$FRONTEND_PID" ]; then
        kill $FRONTEND_PID 2>/dev/null || true
    fi
    
    # 清理进程
    pkill -f "simple_backend" 2>/dev/null || true
    pkill -f "vite" 2>/dev/null || true
    
    echo "✅ 服务已关闭"
    exit 0
}

# 显示状态
show_status() {
    echo ""
    echo "🎉 DuckHub 服务状态"
    echo "==================="
    echo ""
    
    # 检查后端
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        echo "✅ 后端服务: http://localhost:8080"
        echo "   📊 系统信息: http://localhost:8080/info"
        echo "   🏥 健康检查: http://localhost:8080/health"
        echo "   📈 指标数据: http://localhost:8080/metrics"
    else
        echo "❌ 后端服务: 未响应"
    fi
    
    # 检查前端
    if curl -s http://localhost:3000 > /dev/null 2>&1; then
        echo "✅ 前端应用: http://localhost:3000"
    else
        echo "❌ 前端应用: 未响应"
    fi
    
    echo ""
    echo "🔧 API 测试:"
    echo "curl http://localhost:8080/health"
    echo "curl -X POST http://localhost:8080/api/query -H 'Content-Type: application/json' -d '{\"sql\":\"SELECT 1 as test\"}'"
    echo ""
}

# 设置信号处理
trap cleanup SIGINT SIGTERM

# 主流程
main() {
    check_deps
    start_simple_backend
    start_frontend
    show_status
    
    echo "按 Ctrl+C 停止所有服务"
    
    # 保持运行
    while true; do
        sleep 1
    done
}

# 运行主流程
main
