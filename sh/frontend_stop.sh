#!/bin/bash

# --- 配置区 ---
FRONTEND_DIR="../frontend" # 修正后的相对路径
PORT=3000                 # Next.js 默认端口
LOG_FILE="frontend.log"

echo "🛑 正在停止前端进程..."

# 1. 优先通过端口关闭
WEB_PID=$(lsof -t -i:$PORT)

if [ ! -z "$WEB_PID" ]; then
    echo "清理端口 $PORT 对应的进程 (PID: $WEB_PID)..."
    kill $WEB_PID
    sleep 1
fi

# 2. 补刀：清理残留的 Node 开发进程
REMAINING_NODE=$(pgrep -f "node.*dev")
if [ ! -z "$REMAINING_NODE" ]; then
    echo "清理残留的 Node 开发进程..."
    pkill -f "node.*dev"
    sleep 1
fi

# 3. 删除日志文件 (新增部分)
echo "正在清理日志文件..."
if [ -f "$FRONTEND_DIR/$LOG_FILE" ]; then
    rm "$FRONTEND_DIR/$LOG_FILE"
    echo "✅ 已删除 $FRONTEND_DIR/$LOG_FILE"
else
    echo "⚠️ 未发现日志文件，无需清理。"
fi

echo "✨ 前端已关闭并清理完成。"