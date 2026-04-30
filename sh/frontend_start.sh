#!/bin/bash

# --- 配置区 ---
FRONTEND_DIR="../frontend"
LOG_FILE="frontend.log"
PORT=3000 # Vite 默认端口，如果是其他端口请修改

echo "🌐 正在准备启动前端开发服务器..."

# 1. 进入目录
cd $FRONTEND_DIR || { echo "❌ 错误: 找不到 $FRONTEND_DIR 目录"; exit 1; }

# 2. 检查端口占用
if lsof -Pi :$PORT -sTCP:LISTEN -t >/dev/null ; then
    echo "⚠️ 发现端口 $PORT 已被占用，正在清理旧进程..."
    fuser -k $PORT/tcp
    sleep 1
fi

# 3. 启动 npm
echo "正在执行 npm run dev (日志将写入 $FRONTEND_DIR/$LOG_FILE)..."
# 使用 nohup 启动，确保进程在后台持续运行
nohup npm run dev > $LOG_FILE 2>&1 &

# 4. 确认状态
sleep 4
if lsof -Pi :$PORT -sTCP:LISTEN -t >/dev/null ; then
    echo "✅ 前端启动成功！"
    echo "访问地址: http://localhost:$PORT"
    echo "实时日志: tail -f $FRONTEND_DIR/$LOG_FILE"
else
    echo "❌ 前端启动失败，请检查 $FRONTEND_DIR/$LOG_FILE"
fi