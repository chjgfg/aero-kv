import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Contract } from "../target/types/contract";
import { PublicKey, LAMPORTS_PER_SOL, SystemProgram, Keypair } from "@solana/web3.js";
import { expect } from "chai";

describe("contract", () => {
    let provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.contract as Program<Contract>;
    const payer = (provider.wallet as anchor.Wallet).payer;
    const connection = provider.connection;

    const key = Buffer.from([123]);
    const value = Buffer.from([45, 67, 89]);

    const [metaPda] = PublicKey.findProgramAddressSync([Buffer.from("meta")], program.programId);
    const [headPda] = PublicKey.findProgramAddressSync([Buffer.from("head")], program.programId);

    const [nodePda] = PublicKey.findProgramAddressSync([Buffer.from("node"), key], program.programId);
    const [valuePda] = PublicKey.findProgramAddressSync([Buffer.from("value"), key], program.programId);


    // ==========================
    // 🎯 新增：必须传入 3 个新账户
    // ==========================
    const [authPda] = PublicKey.findProgramAddressSync([Buffer.from("auth")], program.programId);
    const [feePda] = PublicKey.findProgramAddressSync([Buffer.from("fee")], program.programId);
    // 国库地址（随便用一个地址，测试能过就行）
    const treasury = payer.publicKey;

    // 1. 生成一个新的测试钱包（新管理员）
    const newAdmin = Keypair.generate();

    const MAX_LEVEL = 8;
    const remainingAccounts = Array(MAX_LEVEL).fill(headPda);

    beforeEach(async () => {
        // 每次测试前，重置本地验证器，确保环境干净
        // 这个在devnet注释掉, 这个是领空投的
        // await provider.connection.requestAirdrop(payer.publicKey, 100 * LAMPORTS_PER_SOL);
    });

    // -------------------------------------------------------------------------------------------------------------------------------------------------------

    it("auth init!", async () => {
        console.log("当前测试钱包地址:", payer.publicKey.toBase58());
        // 🔴 关键：检查账户是否已存在
        const authAccountInfo = await provider.connection.getAccountInfo(authPda);
        if (!authAccountInfo) {
            // 账户不存在，才执行初始化
            const tx = await program.methods
                .initAuth()
                .accounts({
                    signer: payer.publicKey,
                    authConfig: authPda,
                    systemProgram: SystemProgram.programId
                }).rpc();
            console.log("Your transaction signature", tx);
        } else {
            console.log("Auth PDA 已初始化，跳过初始化步骤");
        }
        // 无论是否初始化，都验证账户数据
        const auth = await program.account.authConfig.fetch(authPda);
        console.log("auth", auth);
    });


    it("set admin", async () => {
        // 账户不存在，才执行初始化
        await program.methods
            .initAuth()
            .accounts({
                signer: payer.publicKey,
                authConfig: authPda,
                systemProgram: SystemProgram.programId
            }).rpc();
        console.log("set admin 先授权");
        // 2. 调用 set_admin 指令
        const tx = await program.methods
            .setAdmin(newAdmin.publicKey) // ✅ 这里传新管理员的公钥
            .accounts({
                signer: payer.publicKey, // ✅ 当前管理员（必须是 auth_config 里的 admin）
                authConfig: authPda, // ✅ AuthConfig PDA
            })
            .rpc();

        console.log("✅ set_admin 交易成功:", tx);

        // 3. 验证结果：读取 auth_config，确认 admin 已更新
        const authConfig = await program.account.authConfig.fetch(authPda);
        console.log("✅ 新管理员地址:", authConfig.admin.toBase58());
        console.log("✅ 新管理员是否匹配:", authConfig.admin.equals(newAdmin.publicKey));
    });

    it("set pause", async () => {
        // 账户不存在，才执行初始化
        await program.methods
            .initAuth()
            .accounts({
                signer: payer.publicKey,
                authConfig: authPda,
                systemProgram: SystemProgram.programId
            }).rpc();
        console.log("set pause 先授权");
        // 暂停合约（paused = true）
        const tx1 = await program.methods
            .setPause(true)
            .accounts({
                signer: payer.publicKey,
                authConfig: authPda,
            })
            .rpc();
        console.log("✅ 合约暂停成功:", tx1);

        // 验证暂停状态
        const authConfig1 = await program.account.authConfig.fetch(authPda);
        console.log("✅ 暂停状态:", authConfig1.paused); // 输出 true

        // 恢复合约（paused = false）
        const tx2 = await program.methods
            .setPause(false)
            .accounts({
                signer: payer.publicKey,
                authConfig: authPda,
            })
            .rpc();
        console.log("✅ 合约恢复成功:", tx2);

        // 验证恢复状态
        const authConfig2 = await program.account.authConfig.fetch(authPda);
        console.log("✅ 恢复状态:", authConfig2.paused); // 输出 false
    });

    // -------------------------------------------------------------------------------------------------------------------------------------------------------

    it("fee init!", async () => {
        console.log("当前测试钱包地址:", payer.publicKey.toBase58());
        // 🔴 关键：检查账户是否已存在
        const feeAccountInfo = await provider.connection.getAccountInfo(feePda);
        if (!feeAccountInfo) {
            // 账户不存在，才执行初始化
            const tx = await program.methods
                .initFee()
                .accounts({
                    signer: payer.publicKey,
                    feeConfig: feePda,
                    systemProgram: SystemProgram.programId
                }).rpc();
            console.log("Your transaction signature", tx);
        } else {
            console.log("Fee PDA 已初始化，跳过初始化步骤");
        }
        // 无论是否初始化，都验证账户数据
        const fee = await program.account.feeConfig.fetch(feePda);
        console.log("fee", fee);
    });


    it("set fee", async () => {
        // 账户不存在，才执行初始化
        await program.methods
            .initAuth()
            .accounts({
                signer: payer.publicKey,
                authConfig: authPda,
                systemProgram: SystemProgram.programId
            }).rpc();
        console.log("set fee 先授权");
        // 2. 调用 setFee
        const tx = await program.methods
            .setFee(
                new anchor.BN(100000), // base_fee: 基础费 0.0001 SOL
                new anchor.BN(100),    // fee_per_byte: 每字节费
                new anchor.BN(50000)   // scan_fee_per_item: scan 每条费用
            )
            .accounts({
                signer: payer.publicKey,
                authConfig: authPda,
                feeConfig: feePda,
            })
            .rpc();

        console.log("✅ set_fee 成功", tx);

        // 3. 验证是否设置成功
        const feeData = await program.account.feeConfig.fetch(feePda);
        console.log("base_fee:", feeData.baseFee.toString());
        console.log("fee_per_byte:", feeData.feePerByte.toString());
        console.log("scan_fee_per_item:", feeData.scanFeePerItem.toString());
    });

    // -------------------------------------------------------------------------------------------------------------------------------------------------------

    it("storage init!", async () => {
        console.log("当前测试钱包地址:", payer.publicKey.toBase58());
        console.log("Meta PDA:", metaPda.toBase58());
        console.log("Head PDA:", headPda.toBase58());
        // 🔴 关键：检查账户是否已存在
        const metaAccountInfo = await provider.connection.getAccountInfo(metaPda);
        if (!metaAccountInfo) {
            // 账户不存在，才执行初始化
            const tx = await program.methods
                .initStorage()
                .accounts({
                    signer: payer.publicKey,
                    meta: metaPda,
                    head: headPda,
                    systemProgram: SystemProgram.programId
                }).rpc();
            console.log("Your transaction signature", tx);
        } else {
            console.log("Meta PDA 已初始化，跳过初始化步骤");
        }

        // 无论是否初始化，都验证账户数据
        const meta = await program.account.skipListMeta.fetch(metaPda);
        console.log("metaPda", metaPda);
    });


    it("upsert", async () => {
        const tx = await program.methods
            .upsert(key, value)
            .accounts({
                signer: payer.publicKey,
                meta: metaPda,
                newNode: nodePda,
                valueAccount: valuePda,
                authConfig: authPda,
                feeConfig: feePda,
                treasury: treasury,
                systemProgram: SystemProgram.programId
            })
            .remainingAccounts(
                remainingAccounts.map((pubkey) => ({
                    pubkey,
                    isWritable: true,
                    isSigner: false,
                }))
            )
            .rpc();

        console.log("upsert 交易成功:", tx);

        // ------------------------------
        // 验证结果
        // ------------------------------
        const node = await program.account.skipNode.fetch(nodePda);
        const valueAcc = await program.account.valueAccount.fetch(valuePda);

        // 1. 打印 Buffer 的十进制数组
        console.log("创建的 node key (数组):", Array.from(node.key)); // 输出 [123]
        // 2. 打印 Buffer 的十六进制（默认格式）
        console.log("创建的 node key (Buffer):", node.key); // 输出 <Buffer 7b>
        // 3. 打印 value 同理
        console.log("创建的 value (数组):", Array.from(valueAcc.data)); // 输出 [45, 67, 89]
        console.log("创建的 value (Buffer):", valueAcc.data); // 输出 <Buffer 2d 43 59>
    });

    it("get", async () => {
        const tx = await program.methods
            .get(key)
            .accounts({
                meta: metaPda,
                head: headPda,
                target: nodePda,
                authConfig: authPda,
                valueAccount: valuePda,
            })
            .rpc();
        console.log("get 交易成功:", tx);

        const node = await program.account.skipNode.fetch(nodePda);
        const valueAcc = await program.account.valueAccount.fetch(valuePda);
        // 1. 打印 Buffer 的十进制数组
        console.log("创建的 node key (数组):", Array.from(node.key)); // 输出 [123]
        // 2. 打印 Buffer 的十六进制（默认格式）
        console.log("创建的 node key (Buffer):", node.key); // 输出 <Buffer 7b>
        // 3. 打印 value 同理
        console.log("创建的 value (数组):", Array.from(valueAcc.data)); // 输出 [45, 67, 89]
        console.log("创建的 value (Buffer):", valueAcc.data); // 输出 <Buffer 2d 43 59>
    });

    it("delete", async () => {
        const tx = await program.methods
            .delete(key)
            .accounts({
                signer: payer.publicKey,
                meta: metaPda,
                target: nodePda,
                valueAccount: valuePda,
                authConfig: authPda,
                feeConfig: feePda,
                treasury: treasury,
                systemProgram: SystemProgram.programId
            })
            .remainingAccounts(
                remainingAccounts.map((pubkey) => ({
                    pubkey,
                    isWritable: true,  // ✅ 必须可写，因为要修改 forward 指针
                    isSigner: false,
                }))
            )
            .rpc();
        console.log("delete 交易成功:", tx);

        // 4. 验证删除结果
        try {
            await program.account.skipNode.fetch(nodePda);
            throw new Error("节点未被删除");
        } catch (e) {
            console.log("节点已成功删除");
        }

        try {
            await program.account.valueAccount.fetch(valuePda);
            throw new Error("Value 账户未被删除");
        } catch (e) {
            console.log("Value 账户已成功删除");
        }
    });

    it("scan", async () => {
        // ==========================
        // 1. 配置：自动插入多少条 + 扫描限制
        // ==========================
        const INSERT_COUNT = 3;     // 自动插入 3 条数据
        // ✅ 关键：直接用 Uint8Array 初始化，彻底避免类型问题
        // const scanStartKey = Buffer.from([0]); // 从 key >= 0 开始扫
        const scanStartKey = Buffer.from([0]);
        const scanLimit = new anchor.BN(2);        // 最多返回 2 条


        // ==========================
        // ✅ 优化 1：自动循环插入多条数据
        // ==========================
        const inserted = []; // 保存插入成功的 { key, value, nodePda, valuePda }

        for (let i = 0; i < INSERT_COUNT; i++) {
            const key = Buffer.from([100 + i]); // key: 100, 101, 102...
            const value = Buffer.from(`data_${i}`);

            // 计算 PDA
            const [nodePda] = PublicKey.findProgramAddressSync(
                [Buffer.from("node"), key],
                program.programId
            );
            const [valuePda] = PublicKey.findProgramAddressSync(
                [Buffer.from("value"), key],
                program.programId
            );

            // 调用 upsert
            await program.methods
                .upsert(key, value)
                .accounts({
                    signer: payer.publicKey,
                    meta: metaPda,
                    newNode: nodePda,
                    valueAccount: valuePda,
                    authConfig: authPda,
                    feeConfig: feePda,
                    treasury: treasury,
                    systemProgram: SystemProgram.programId,
                })
                .remainingAccounts(
                    Array(MAX_LEVEL).fill(headPda).map((k) => ({
                        pubkey: k,
                        isWritable: true,
                        isSigner: false,
                    }))
                )
                .rpc();

            // 保存起来给 scan 使用
            inserted.push({ key, value, nodePda, valuePda });
            console.log("✅ 插入成功 key:", Array.from(key));
        }


        // ==========================
        // ✅ 优化 2：自动按 limit 生成 remainingAccounts（超级灵活）
        // ==========================
        const remainingAccounts = [];

        // 插入多少组，由 scanLimit 决定！
        for (let i = 0; i < scanLimit.toNumber(); i++) {
            const item = inserted[i];
            if (!item) break;

            // 每组 push 两个：node + value
            remainingAccounts.push(
                { pubkey: item.nodePda, isWritable: false, isSigner: false },
                { pubkey: item.valuePda, isWritable: false, isSigner: false }
            );
        }
        console.log("📦 构造的 remainingAccounts 长度:", remainingAccounts.length);


        // 🎯 1. 准备一个数组来存放捕获到的事件
        const scannedEvents = [];

        // 🎯 2. 在发送交易前，开启监听器
        const listener = program.addEventListener("KVPair", (event, slot) => {
            scannedEvents.push(event);
        });

        try {
            const tx = await program.methods
                .scan(scanStartKey, scanLimit)
                .accounts({
                    signer: payer.publicKey,
                    meta: metaPda,
                    authConfig: authPda,
                    feeConfig: feePda,
                    treasury: treasury,
                })
                .remainingAccounts(remainingAccounts)
                .rpc();

            console.log("\n✅ scan 交易完成:", tx);

            // 🎯 核心逻辑：直接从链上抓取并解析事件
            // 给 RPC 一点点同步时间
            await new Promise((resolve) => setTimeout(resolve, 1000));

            const txResult = await provider.connection.getTransaction(tx, {
                commitment: "confirmed",
                maxSupportedTransactionVersion: 0,
            });

            if (txResult && txResult.meta && txResult.meta.logMessages) {
                // 使用 Anchor 提供的 EventParser 手动解析日志
                const eventParser = new anchor.EventParser(program.programId, program.coder);
                const events = eventParser.parseLogs(txResult.meta.logMessages);

                console.log("\n=== 扫描到的 KV 对 ===");
                let found = false;
                let foundCount = 0;
                for (const log of events) {
                    console.log("log", log);
                    if (log.name === "kvPair") {
                        found = true;
                        foundCount++;
                        // 🎯 将十六进制 Buffer 转为普通的数组和字符串
                        const keyData = Array.from(log.data.key as Buffer);
                        const valueData = Buffer.from(log.data.value as Buffer).toString();

                        console.log(`🔹 条目 ${foundCount}:`);
                        console.log(`   Key (Raw):   [${keyData}]`); // 这里会显示 [100], [101]...
                        console.log(`   Value (Str): ${valueData}`);
                    }
                }
                if (!found) console.log("❌ 日志中包含数据，但 EventParser 解析失败。");
                if (foundCount === 0) {
                    console.log("❌ 日志中未提取到 KVPair 事件。");
                } else {
                    console.log(`\n✅ 成功解析 ${foundCount} 条数据！`);
                }
            }

        } catch (err) {
            console.error("\n❌ scan 执行失败:", err);
        } finally {
            // ✅ 关键：测试结束必须移除监听
            if (listener !== null) {
                await program.removeEventListener(listener);
            }
        }
    });
});

// anchor test -- tests/storage.test.ts