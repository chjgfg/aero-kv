#!/bin/bash

# --- 配置区 ---
LEDGER_DIR="test-ledger"
PID_FILE=".validator.pid"
LOG_FILE="validator.log"

if [ -f "$PID_FILE" ]; then
    echo "停止 solana-test-validator..."
    kill $(cat "$PID_FILE") && rm "$PID_FILE"
    sleep 1
else
    # 如果 PID 文件不存在，尝试通过端口强制关闭
    fuser -k 8899/tcp || true
fi

# 1. 删除账本数据 (对应图中 test-ledger)
echo "正在清理测试账本数据..."
if [ -d "$LEDGER_DIR" ]; then
    rm -rf "$LEDGER_DIR"
    echo "✅ 已删除 $LEDGER_DIR"
fi

# 2. 删除日志文件 (对应图中 validator.log)
echo "正在清理测试链日志..."
if [ -f "$LOG_FILE" ]; then
    rm "$LOG_FILE"
    echo "✅ 已删除 $LOG_FILE"
fi

echo "✅ 测试链已停止并清理完成"