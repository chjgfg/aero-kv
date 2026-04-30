#!/bin/bash
set -euo pipefail

# 配置
CONTRACT_DIR="../contract"
VALIDATOR_PORT=8899

# 1. 清理旧进程（占用8899端口的 validator）
echo "🧹 清理旧的 solana-test-validator 进程..."
if lsof -i :$VALIDATOR_PORT >/dev/null 2>&1; then
    fuser -k $VALIDATOR_PORT/tcp || true
    sleep 1
fi

# 2. 开启 io_uring（解决 validator 报错）
echo "⚙️  配置 kernel.io_uring_disabled=0..."
sysctl -w kernel.io_uring_disabled=0

# 3. 启动 solana-test-validator（后台运行，输出到日志）
echo "🚀 启动 solana-test-validator..."
solana-test-validator -r > validator.log 2>&1 &
VALIDATOR_PID=$!
echo "validator PID: $VALIDATOR_PID"
echo $VALIDATOR_PID > .validator.pid

# 等待 validator 启动完成
echo "⏳ 等待 validator 初始化..."
sleep 5
for i in {1..30}; do
    if curl -s http://127.0.0.1:$VALIDATOR_PORT >/dev/null; then
        echo "✅ validator 已就绪"
        break
    fi
    echo "等待中... ($i/30)"
    sleep 2
done

# 4. 构建并部署合约
echo "🔨 构建合约..."
cd "$CONTRACT_DIR"
anchor build

echo "📦 部署合约到本地链..."
anchor deploy

echo "✅ 合约部署成功！Program ID 如下："
grep "Program Id" target/deploy/contract-keypair.json || echo "请查看上面的输出获取 Program ID"

echo ""
echo "📋 后续操作提示："
echo "  - 查看 validator 日志：tail -f validator.log"
echo "  - 停止 validator：kill $(cat .validator.pid) && rm .validator.pid"