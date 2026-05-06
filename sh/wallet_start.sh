#!/bin/bash

# 1. 创建并进入测试文件夹
mkdir -p ../test_wallets && cd ../test_wallets

# 2. 清理旧的记录文件
rm -f pubkeys.txt

echo "开始批量生成 5 个测试钱包并申请空投..."

# 3. 批量生成钱包、提取公钥并申请空投
for i in {1..5}; do
    # 生成钱包文件
    solana-keygen new --no-passphrase -so "wallet_$i.json" > /dev/null
    
    # 提取公钥
    pubkey=$(solana-keygen pubkey "wallet_$i.json")
    
    # 🌟 新增：申请 50 SOL 空投
    # 注意：这需要你的本地 solana-test-validator 正在运行
    echo "正在为 Wallet $i ($pubkey) 申请 50 SOL..."
    solana airdrop 50 "$pubkey" --url localhost > /dev/null
    
    # 追加到记录文件
    echo "Wallet $i: $pubkey" >> pubkeys.txt
    
    echo "已完成第 $i 个: $pubkey (已申请空投)"
done

echo "-----------------------------------------------"
echo "✅ 所有公钥已保存至: ../test_wallets/pubkeys.txt"
echo "💰 每个账户已尝试申请 50 SOL 空投"
echo "你可以通过 solana balance <PUBKEY> --url localhost 来验证余额"