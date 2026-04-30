#!/bin/bash

# --- 配置区 ---
BACKEND_DIR="../backend"
BINARY_NAME="backend"

echo "正在关闭所有 $BINARY_NAME 进程..."
pkill -x $BINARY_NAME

# 等待进程完全退出
sleep 1

echo "正在清理持久化数据..."
# 检查当前目录下是否有 backend 文件夹，或者是否已经在 backend 里面
if [ -d "$BACKEND_DIR/kv" ]; then
    rm -rf "$BACKEND_DIR/kv"
    echo "✅ 已清理 $BACKEND_DIR/kv"
elif [ -d "kv" ]; then
    rm -rf kv
    echo "✅ 已清理当前目录下的 kv"
else
    echo "⚠️ 未发现 kv 文件夹，无需清理。"
fi

# 顺便清理一下生成的 nohup 日志文件（可选）
rm $BACKEND_DIR/*.log 2>/dev/null

echo "✨ 停止并清理完成。"