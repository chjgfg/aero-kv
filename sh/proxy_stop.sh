#!/bin/bash

# --- 配置区 ---
PROXY_DIR="../proxy"   # 修正后的相对路径
PROXY_PORT=80
LOG_FILE="proxy.log"

echo "🛑 正在停止 Proxy..."

# 1. 尝试通过端口查找并杀死进程
PROXY_PID=$(lsof -t -i:$PROXY_PORT)

if [ -z "$PROXY_PID" ]; then
    # 如果端口没查到，尝试通过进程名查找
    echo "未在端口 $PROXY_PORT 发现进程，尝试搜索进程名..."
    PROXY_PID=$(pgrep -f "target/debug/proxy")
fi

if [ ! -z "$PROXY_PID" ]; then
    echo "发现 Proxy 进程 (PID: $PROXY_PID)，正在关闭..."
    kill $PROXY_PID
    sleep 1
    echo "✅ Proxy 已关闭。"
else
    echo "⚠️ 未发现正在运行的 Proxy 进程。"
fi

# 2. 删除日志文件 (新增部分)
echo "正在清理代理日志..."
if [ -f "$PROXY_DIR/$LOG_FILE" ]; then
    rm "$PROXY_DIR/$LOG_FILE"
    echo "✅ 已删除 $PROXY_DIR/$LOG_FILE"
else
    echo "⚠️ 未发现代理日志文件，无需清理。"
fi

echo "✨ Proxy 停止并清理完成。"