#!/bin/bash

# DuckHub 开发环境快速启动脚本

set -e

echo "🚀 DuckHub 开发环境启动"
echo "========================"

# 检查是否在项目根目录
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 请在项目根目录运行此脚本"
    exit 1
fi

# 函数：启动后端
start_backend() {
    echo "📡 启动后端服务..."
    
    # 设置环境变量
    export RUST_LOG=info
    export DUCKHUB_HOST=0.0.0.0
    export DUCKHUB_PORT=8080
    
    # 启动后端服务（开发模式）
    cargo run --bin duckhub-web-api &
    BACKEND_PID=$!
    
    echo "后端服务 PID: $BACKEND_PID"
    echo "等待后端服务启动..."
    sleep 8
    
    # 检查后端是否启动成功
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        echo "✅ 后端服务启动成功: http://localhost:8080"
    else
        echo "⚠️  后端服务可能还在启动中..."
    fi
}

# 函数：启动前端
start_frontend() {
    echo "🌐 启动前端服务..."
    
    cd crates/web-frontend
    
    # 检查依赖
    if [ ! -d "node_modules" ]; then
        echo "📦 安装前端依赖..."
        npm install
    fi
    
    # 设置环境变量
    export VITE_API_URL=http://localhost:8080
    
    # 启动前端开发服务器
    npm run dev &
    FRONTEND_PID=$!
    
    cd ../..
    
    echo "前端服务 PID: $FRONTEND_PID"
    echo "等待前端服务启动..."
    sleep 5
    
    echo "✅ 前端服务启动成功: http://localhost:3000"
}

# 清理函数
cleanup() {
    echo ""
    echo "🛑 正在关闭服务..."
    
    if [ ! -z "$BACKEND_PID" ]; then
        kill $BACKEND_PID 2>/dev/null || true
    fi
    
    if [ ! -z "$FRONTEND_PID" ]; then
        kill $FRONTEND_PID 2>/dev/null || true
    fi
    
    # 清理可能的进程
    pkill -f "duckhub-web-api" 2>/dev/null || true
    pkill -f "vite" 2>/dev/null || true
    
    echo "✅ 服务已关闭"
    exit 0
}

# 设置信号处理
trap cleanup SIGINT SIGTERM

# 主启动流程
echo "🔧 检查 Rust 环境..."
if ! command -v cargo &> /dev/null; then
    echo "❌ 请先安装 Rust: https://rustup.rs/"
    exit 1
fi

echo "🔧 检查 Node.js 环境..."
if ! command -v node &> /dev/null; then
    echo "❌ 请先安装 Node.js 18+: https://nodejs.org/"
    exit 1
fi

# 启动服务
start_backend
start_frontend

echo ""
echo "🎉 DuckHub 开发环境启动完成！"
echo ""
echo "📍 服务地址:"
echo "   🌐 前端应用: http://localhost:3000"
echo "   📡 后端 API: http://localhost:8080"
echo "   🏥 健康检查: http://localhost:8080/health"
echo "   📊 指标监控: http://localhost:8080/metrics"
echo ""
echo "🔧 功能模块:"
echo "   🏠 仪表板: http://localhost:3000/"
echo "   📊 数据查询: http://localhost:3000/query"
echo "   🗃️ DuckLake: http://localhost:3000/ducklake"
echo "   🤖 AI 助手: http://localhost:3000/ai"
echo ""
echo "按 Ctrl+C 停止所有服务"

# 保持脚本运行
while true; do
    sleep 1
done
