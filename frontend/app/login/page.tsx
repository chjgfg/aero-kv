"use client";

import React, { useState } from 'react';
import { LogIn, Key } from 'lucide-react';
import { login } from '@/utils/http';
import { useRouter } from "next/navigation";

export default function LoginPage() {
    const [pubkey, setPubkey] = useState("");
    const router = useRouter();

    const handleLogin = async () => {
        console.log("pubkey:", pubkey);
        console.log("pubkey.length:", pubkey.length);
        if (pubkey.length < 32) {
            alert("请输入正确的公钥");
            return;
        }
        await login(pubkey);
        // 调用你后端 SessionManager 的 login 接口
        console.log("正在登录公钥:", pubkey);
        // router.push("/permission");
        router.push("/dashboard");
    };

    return (
        <div className="min-h-screen bg-slate-950 flex items-center justify-center p-4">
            <div className="w-full max-w-md bg-slate-900 border border-slate-800 rounded-3xl p-8 shadow-2xl">
                <h1 className="text-2xl font-bold text-white mb-8 flex items-center gap-3">
                    <LogIn className="text-cyan-500" /> 系统登录
                </h1>

                <div className="space-y-6">
                    <div className="relative">
                        <Key className="absolute left-4 top-4 text-slate-500" size={20} />
                        <input
                            type="text"
                            value={pubkey}
                            onChange={(e) => setPubkey(e.target.value)}
                            placeholder="请输入您的公钥 (Base58)"
                            className="w-full bg-slate-950 border border-slate-700 rounded-xl pl-12 pr-4 py-4 text-white focus:border-cyan-500 outline-none transition-all font-mono text-sm"
                        />
                    </div>

                    <button
                        onClick={handleLogin}
                        className="w-full bg-cyan-600 hover:bg-cyan-500 text-white font-bold py-4 rounded-xl transition-all active:scale-[0.98]"
                    >
                        立即登录
                    </button>
                </div>
            </div>
        </div>
    );
}