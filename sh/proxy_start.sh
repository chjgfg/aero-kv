#!/bin/bash

# --- 配置区 ---
PROXY_DIR="../proxy"
PROXY_PORT=80
LOG_FILE="proxy.log"

echo "🚀 正在准备启动 Proxy..."

# 1. 检查 80 端口是否被占用
if lsof -Pi :$PROXY_PORT -sTCP:LISTEN -t >/dev/null ; then
    echo "⚠️ 错误: 端口 $PROXY_PORT 已被占用，正在尝试关闭旧进程..."
    fuser -k $PROXY_PORT/tcp
    sleep 1
fi

# 2. 进入 proxy 目录
cd $PROXY_DIR || { echo "❌ 错误: 找不到 $PROXY_DIR 目录"; exit 1; }

# 3. 启动代理
echo "正在使用 cargo run 启动代理 (日志将写入 $PROXY_DIR/$LOG_FILE)..."
# 使用 nohup 确保关闭终端后代理继续运行
nohup cargo run > $LOG_FILE 2>&1 &

# 4. 确认启动状态
sleep 3
if pgrep -f "target/debug/proxy" > /dev/null; then
    echo "✅ Proxy 启动成功！"
    echo "你可以运行 'tail -f $PROXY_DIR/$LOG_FILE' 查看实时日志。"
    echo "当前代理状态:"
    curl -s http://127.0.0.1:80/proxy/status || echo "暂时无法获取状态（可能后端未就绪）"
else
    echo "❌ Proxy 启动失败，请检查 $PROXY_DIR/$LOG_FILE"
fi