"use client";

import { useWallet } from "@solana/wallet-adapter-react";
import { useRouter } from "next/navigation";
import dynamic from "next/dynamic"; // 引入动态加载工具
import { login } from "@/utils/http";

// 1. 动态导入 WalletMultiButton，并关闭服务端渲染
const WalletMultiButtonDynamic = dynamic(
    async () => (await import("@solana/wallet-adapter-react-ui")).WalletMultiButton,
    { ssr: false }
);

export default function LoginPage() {
    const { connected, publicKey, disconnect } = useWallet();
    const router = useRouter();

    const handleLoginClick = async () => {
        if (publicKey) {
            // 将公钥转成 Base58 字符串传给后端工具函数
            await login(publicKey.toBase58());
            router.push("/dashboard");
        }
    };

    return (
        <div className="min-h-screen bg-[#0f172a] flex flex-col items-center justify-center px-4">
            <div className="w-full max-w-md bg-[#1e293b] rounded-2xl p-8 shadow-xl">
                <h1 className="text-3xl font-bold text-center text-blue-400 mb-2">
                    Aero KV 管理后台
                </h1>
                <p className="text-gray-400 text-center mb-8">
                    连接 Solana 钱包即可登录
                </p>

                <div className="flex justify-center mb-6">
                    {/* 2. 使用动态加载的按钮组件 */}
                    <WalletMultiButtonDynamic />
                </div>

                {connected && publicKey && (
                    <div className="space-y-4">
                        <div className="p-4 bg-gray-800 rounded-lg text-center">
                            <p className="text-sm text-gray-400">已连接钱包</p>
                            <p className="text-cyan-300 font-mono text-sm break-all">
                                {publicKey.toBase58()}
                            </p>
                        </div>

                        <button
                            onClick={() => router.push("/permission")}
                            // onClick={() => handleLoginClick}
                            className="w-full bg-green-500 hover:bg-green-600 text-white py-4 rounded-xl font-semibold transition"
                        >
                            进入管理后台
                        </button>

                        <button
                            onClick={disconnect}
                            className="w-full py-3 text-gray-400 hover:text-white text-sm transition"
                        >
                            断开连接
                        </button>
                    </div>
                )}
            </div>
        </div>
    );
}