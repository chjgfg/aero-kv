#!/bin/bash

# --- 配置区 ---
BACKEND_DIR="../backend"
BINARY_NAME="backend"
NODES=(1 2 3)
PORTS=(8001 8002 8003)

echo "[1/4] 正在清理旧进程和数据..."
pkill -x $BINARY_NAME 2>/dev/null

# 进入 backend 目录
cd $BACKEND_DIR || { echo "❌ 错误: 找不到 $BACKEND_DIR 目录"; exit 1; }

# 删除 kv 文件夹及其下的所有日志和数据
if [ -d "kv" ]; then
    rm -rf kv
    echo "清理 kv 数据目录完成"
fi
sleep 1

echo "[2/4] 正在编译程序..."
cargo build
if [ $? -ne 0 ]; then
    echo "❌ 编译失败，请检查代码！"
    exit 1
fi

echo "[3/4] 启动 3 个节点..."
for i in {0..2}; do
    ID=${NODES[$i]}
    PORT=${PORTS[$i]}
    echo "启动 Node $ID (Port $PORT)..."
    nohup ./target/debug/$BINARY_NAME $ID $PORT > "node_$ID.log" 2>&1 &
done

echo "等待节点就绪 (6秒)..."
sleep 6

echo "[4/4] 正在初始化集群..."
curl -X POST http://127.0.0.1:8001/raft/init \
     -H "Content-Type: application/json" \
     -d '{
        "1": { "addr": "127.0.0.1:8001" },
        "2": { "addr": "127.0.0.1:8002" },
        "3": { "addr": "127.0.0.1:8003" }
     }'

echo -e "\n✅ 集群启动完成！"