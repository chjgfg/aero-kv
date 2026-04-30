#!/bin/bash

# --- 配置区 ---
FRONTEND_DIR="../frontend"
PORT=3000
LOG_FILE="frontend.log"

echo "🛑 正在强力清理前端环境..."

# 1. 真正的“斩草除根”：直接让内核切断 3000 端口的所有连接并杀死相关进程
# 加上 sudo 是为了防止权限不足查不到进程
if command -v fuser >/dev/null 2>&1; then
    echo "使用 fuser 强制释放端口 $PORT..."
    fuser -k -9 $PORT/tcp >/dev/null 2>&1
else
    # 如果没有 fuser，改用更暴力的 lsof + kill
    PIDS=$(lsof -t -i:$PORT)
    if [ ! -z "$PIDS" ]; then
        echo "发现进程 $PIDS，正在强制杀死..."
        kill -9 $PIDS >/dev/null 2>&1
    fi
fi

# 2. 针对 Next.js 的“双重保险”：清理可能存在的残留 node 开发进程
# 只针对你前端目录下的 node 进程，绝对不伤 SSH 连接
FRONTEND_PATH=$(realpath "$FRONTEND_DIR")
REMAINING_PIDS=$(ps -ef | grep node | grep "$FRONTEND_PATH" | grep -v grep | awk '{print $2}')
if [ ! -z "$REMAINING_PIDS" ]; then
    echo "清理残留 node 进程: $REMAINING_PIDS"
    kill -9 $REMAINING_PIDS >/dev/null 2>&1
fi

# 3. 清理日志文件
if [ -f "$FRONTEND_DIR/$LOG_FILE" ]; then
    rm "$FRONTEND_DIR/$LOG_FILE"
    echo "✅ 日志已删除。"
fi

# 4. 验证清理结果
sleep 1
if ! lsof -i:$PORT >/dev/null; then
    echo "✨ 成功！端口 $PORT 已完全释放。"
else
    echo "❌ 警告：端口 $PORT 似乎仍被占用，请手动执行 'sudo fuser -k 3000/tcp'"
fi