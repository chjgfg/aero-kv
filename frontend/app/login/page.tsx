"use client";

import React, { useState } from 'react';
import { UserPlus, ShieldCheck, AlertCircle, Copy, Check } from 'lucide-react';

export default function GrantPermission() {
    const [pubkey, setPubkey] = useState("");
    const [isCopied, setIsCopied] = useState(false);

    // 基础校验：Solana 公钥通常是 32-44 位的 Base58 字符串
    const isValidPubkey = pubkey.length >= 32 && pubkey.length <= 44;

    const handleCopy = () => {
        if (pubkey) {
            navigator.clipboard.writeText(pubkey);
            setIsCopied(true);
            setTimeout(() => setIsCopied(false), 2000);
        }
    };

    return (
        <div className="bg-slate-900 border border-slate-800 rounded-3xl p-8 shadow-2xl">
            {/* 标题部分 */}
            <div className="flex items-center gap-3 mb-6">
                <div className="p-3 bg-cyan-500/10 rounded-2xl">
                    <UserPlus className="text-cyan-500" size={24} />
                </div>
                <div>
                    <h2 className="text-xl font-bold text-white">目标用户授权</h2>
                    <p className="text-sm text-slate-400">请输入接收权限的 Solana 钱包公钥</p>
                </div>
            </div>

            {/* 输入区域 */}
            <div className="space-y-4">
                <div className="relative group">
                    <input
                        type="text"
                        value={pubkey}
                        onChange={(e) => setPubkey(e.target.value)}
                        placeholder="例如: 7xKX...vT2p"
                        className={`
              w-full bg-slate-950 border-2 rounded-2xl px-5 py-4 font-mono text-sm transition-all outline-none
              ${pubkey === ""
                                ? "border-slate-800 focus:border-slate-600"
                                : isValidPubkey
                                    ? "border-emerald-500/50 focus:border-emerald-500"
                                    : "border-rose-500/50 focus:border-rose-500"}
            `}
                    />

                    {/* 右侧工具栏 */}
                    <div className="absolute right-3 top-1/2 -translate-y-1/2 flex items-center gap-2">
                        {pubkey && (
                            <button
                                onClick={handleCopy}
                                className="p-2 hover:bg-slate-800 rounded-lg text-slate-500 transition-colors"
                                title="复制地址"
                            >
                                {isCopied ? <Check size={18} className="text-emerald-500" /> : <Copy size={18} />}
                            </button>
                        )}
                    </div>
                </div>

                {/* 校验提示 */}
                <div className="flex items-center gap-2 px-2">
                    {pubkey === "" ? (
                        <div className="flex items-center gap-2 text-slate-500 text-xs">
                            <AlertCircle size={14} />
                            <span>待输入公钥</span>
                        </div>
                    ) : isValidPubkey ? (
                        <div className="flex items-center gap-2 text-emerald-500 text-xs">
                            <ShieldCheck size={14} />
                            <span>格式校验通过，该地址可以接收 Raft 提案</span>
                        </div>
                    ) : (
                        <div className="flex items-center gap-2 text-rose-500 text-xs font-medium">
                            <AlertCircle size={14} />
                            <span>无效的公钥格式，请检查长度 (32-44位)</span>
                        </div>
                    )}
                </div>
            </div>

            {/* 底部快速说明 */}
            <div className="mt-8 p-4 bg-slate-950/50 rounded-2xl border border-slate-800/50">
                <h4 className="text-xs font-bold text-slate-500 uppercase tracking-widest mb-2">注意事项</h4>
                <ul className="text-[11px] text-slate-500 space-y-1 list-disc pl-4">
                    <li>授权操作将作为一条日志写入 Raft 存储层</li>
                    <li>只有 Leader 节点可以发起此项修改提案</li>
                    <li>权限一旦通过共识，将在集群所有节点的内存中即时更新[cite: 1]</li>
                </ul>
            </div>
        </div>
    );
}