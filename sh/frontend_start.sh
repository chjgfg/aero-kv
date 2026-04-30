#!/bin/bash

# --- 配置区 ---
FRONTEND_DIR="../frontend"
LOG_FILE="frontend.log"
PORT=3000 

echo "🌐 正在准备启动前端开发服务器..."

# 1. 进入目录
cd $FRONTEND_DIR || { echo "❌ 错误: 找不到 $FRONTEND_DIR 目录"; exit 1; }

# 2. 检查端口占用
# 这里的清理要彻底，防止旧进程干扰检测
if lsof -Pi :$PORT -sTCP:LISTEN -t >/dev/null ; then
    echo "⚠️ 发现端口 $PORT 已被占用，正在强行清理旧进程..."
    fuser -k $PORT/tcp >/dev/null 2>&1
    pkill -f "node.*dev" >/dev/null 2>&1
    sleep 2 # 给系统释放端口一点缓冲时间
fi

# 3. 启动 npm
echo "正在执行 npm run dev (日志写入 ./$LOG_FILE)..."
# 注意：这里我们已经 cd 进了 frontend，直接写 $LOG_FILE 即可
nohup npm run dev > $LOG_FILE 2>&1 &
# 4. 确认状态 (改用网络探测，比 lsof 更准)
echo "正在等待前端响应 (最多等待 15 秒)..."
MAX_RETRIES=15
COUNT=0
SUCCESS=false

while [ $COUNT -lt $MAX_RETRIES ]; do
    # 使用 curl 尝试静默访问，如果返回状态码(任意)即代表服务已就绪
    if curl -s -o /dev/null http://127.0.0.1:$PORT; then
        SUCCESS=true
        break
    fi
    sleep 1
    COUNT=$((COUNT + 1))
    echo -n "."
done

echo "" # 换行

if [ "$SUCCESS" = true ]; then
    echo "✅ 前端启动成功！"
    echo "访问地址: http://localhost:$PORT"
    # 注意：这里路径要对
    echo "实时日志: tail -f ../frontend/$LOG_FILE"
else
    echo "❌ 依然检测不到端口 $PORT，但进程可能已在后台运行。"
    echo "请手动执行 'lsof -i :$PORT' 确认。"
fi