"use client";

import { useWallet } from "@solana/wallet-adapter-react";
import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";

type HealthResponse = {
    status: string;
    program_id: string;
    message?: string;
};

export default function DashboardPage() {
    const { connected, publicKey, disconnect } = useWallet();
    const router = useRouter();
    const [health, setHealth] = useState<HealthResponse | null>(null);
    const [loading, setLoading] = useState(true);

    // 登录守卫：未连接钱包自动跳回登录
    useEffect(() => {
        if (!connected) {
            router.replace("/login");
        }
    }, [connected, router]);

    // 拉取后端健康状态
    useEffect(() => {
        const fetchHealth = async () => {
            if (!connected) return;
            try {
                // 替换为你的后端地址
                const res = await fetch("http://192.168.40.131/health");
                const data = await res.json();
                console.log(data);
                setHealth(data);
            } catch (err) {
                console.error("健康检查失败:", err);
            } finally {
                setLoading(false);
            }
        };

        fetchHealth();
    }, [connected]);

    // 加载中/未登录状态
    if (!connected) return null;

    return (
        <div className="min-h-screen bg-[#0f172a] text-white">
            {/* 顶部导航 */}
            <header className="bg-[#1e293b] shadow-md px-6 py-4 flex justify-between items-center">
                <h1 className="text-xl font-bold text-blue-400">Aero KV 管理后台</h1>
                <div className="flex items-center gap-4">
                    <p className="text-sm text-gray-300 font-mono">
                        {publicKey?.toBase58().slice(0, 8)}...
                    </p>
                    <button
                        onClick={disconnect}
                        className="text-sm px-3 py-1 bg-red-500 hover:bg-red-600 rounded transition"
                    >
                        退出登录
                    </button>
                </div>
            </header>

            {/* 主内容区 */}
            <main className="max-w-7xl mx-auto px-4 py-8">
                {/* 服务状态卡片 */}
                <div className="bg-[#1e293b] p-6 rounded-2xl shadow mb-8">
                    <h2 className="text-lg font-semibold mb-4">服务状态</h2>
                    {loading ? (
                        <p className="text-gray-400">加载中...</p>
                    ) : health ? (
                        <div className="space-y-2">
                            <div className="flex items-center gap-2">
                                <span className="w-2 h-2 rounded-full bg-green-400"></span>
                                <span>状态：{health.status}</span>
                            </div>
                            <div className="text-sm text-gray-300">
                                程序ID：{health.program_id}
                            </div>
                        </div>
                    ) : (
                        <p className="text-red-400">后端服务连接失败</p>
                    )}
                </div>

                {/* 功能入口卡片 */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <FeatureCard
                        title="KV 数据管理"
                        desc="新增、查询、修改、删除链上 KV 数据"
                        path="/dashboard/kv"
                    />
                    <FeatureCard
                        title="权限管理"
                        desc="设置管理员、暂停/启动合约"
                        path="/dashboard/auth"
                    />
                    <FeatureCard
                        title="手续费管理"
                        desc="初始化手续费、设置费率"
                        path="/dashboard/fee"
                    />
                    <FeatureCard
                        title="数据扫描"
                        desc="扫描所有链上存储数据"
                        path="/dashboard/scan"
                    />
                </div>
            </main>
        </div>
    );
}

// 功能卡片组件
function FeatureCard({
    title,
    desc,
    path,
}: {
    title: string;
    desc: string;
    path: string;
}) {
    const router = useRouter();
    return (
        <div
            onClick={() => router.push(path)}
            className="bg-[#1e293b] p-6 rounded-2xl shadow hover:bg-[#2a3756] cursor-pointer transition"
        >
            <h3 className="text-lg font-bold text-blue-400 mb-2">{title}</h3>
            <p className="text-gray-300 text-sm">{desc}</p>
        </div>
    );
}