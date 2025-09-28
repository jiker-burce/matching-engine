#!/bin/bash

# 撮合引擎启动脚本
echo "🚀 启动撮合引擎系统..."

# 检查是否在正确的目录
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 错误: 请在项目根目录运行此脚本"
    exit 1
fi

# 启动后端服务器
echo "📦 启动后端服务器 (端口 8888)..."
cargo run &
BACKEND_PID=$!

# 等待后端服务器启动
echo "⏳ 等待后端服务器启动..."
sleep 3

# 检查后端是否启动成功
if curl -s http://localhost:8888/health > /dev/null; then
    echo "✅ 后端服务器启动成功"
else
    echo "❌ 后端服务器启动失败"
    kill $BACKEND_PID 2>/dev/null
    exit 1
fi

# 检查前端目录是否存在
if [ ! -d "frontend" ]; then
    echo "❌ 错误: 前端目录不存在"
    kill $BACKEND_PID 2>/dev/null
    exit 1
fi

# 进入前端目录
cd frontend

# 检查node_modules是否存在
if [ ! -d "node_modules" ]; then
    echo "📦 安装前端依赖..."
    npm install
    if [ $? -ne 0 ]; then
        echo "❌ 前端依赖安装失败"
        kill $BACKEND_PID 2>/dev/null
        exit 1
    fi
fi

# 启动前端开发服务器
echo "🌐 启动前端开发服务器 (端口 3300)..."
npm run dev &
FRONTEND_PID=$!

# 等待前端启动
sleep 5

echo ""
echo "🎉 撮合引擎系统启动完成！"
echo ""
echo "📊 后端API: http://localhost:8888"
echo "🌐 前端界面: http://localhost:3300"
echo ""
echo "💡 使用 Ctrl+C 停止所有服务"
echo ""

# 等待用户中断
trap 'echo ""; echo "🛑 正在停止服务..."; kill $BACKEND_PID $FRONTEND_PID 2>/dev/null; echo "✅ 服务已停止"; exit 0' INT

# 保持脚本运行
wait
