"use client";

import { useEffect, useState } from "react";
import Link from "next/link";

export default function VisualPage() {
    const [proofData, setProofData] = useState<any>(null);

    useEffect(() => {
        const data = localStorage.getItem("lastProof");
        if (data) {
            try {
                setProofData(JSON.parse(data));
            } catch (e) {
                console.error("解析缓存数据失败", e);
            }
        }
    }, []);

    if (!proofData) {
        return (
            <div className="min-h-screen bg-gray-900 text-gray-100 flex flex-col items-center justify-center">
                <h1 className="text-3xl font-bold text-purple-400 mb-4">🌳 Merkle 可视化</h1>
                <p className="text-gray-400 mb-8">请先在主页查询数据，生成验证路径</p>
                <Link href="/dashboard" className="px-6 py-2 bg-purple-600 rounded-lg hover:bg-purple-500 transition">返回</Link>
            </div>
        );
    }

    // 兼容逻辑：如果是旧的单条格式，转换为新的多条数组格式
    const leaves = proofData.leaves || [{
        key: proofData.key,
        value: proofData.value,
        hash: proofData.key_hash,
        index: proofData.leaf_index
    }];

    return (
        <div className="min-h-screen bg-[#0f172a] text-gray-100 p-8">
            <div className="max-w-6xl mx-auto">
                <div className="flex justify-between items-center mb-12">
                    <div>
                        <h1 className="text-3xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-purple-400">
                            AeroKV Merkle 范围证明
                        </h1>
                        <p className="text-slate-400 text-sm mt-2">展示 {leaves.length} 条数据的原始完整性证据链</p>
                    </div>
                    <Link href="/dashboard" className="px-4 py-2 bg-slate-800 border border-slate-700 rounded-lg hover:bg-slate-700 transition">
                        返回
                    </Link>
                </div>

                <div className="flex flex-col items-center space-y-10">

                    {/* 1. 叶子节点层：根据数据量自动横向排布 */}
                    <div className="w-full">
                        <h3 className="text-center text-xs font-semibold text-blue-400 uppercase tracking-widest mb-6">数据叶子节点 (Leaves)</h3>
                        <div className="flex flex-wrap justify-center gap-4">
                            {leaves.map((leaf: any, i: number) => (
                                <div key={i} className="group relative">
                                    <div className="bg-blue-900/40 border border-blue-500/50 p-4 rounded-xl w-40 text-center hover:border-blue-400 transition-all shadow-lg">
                                        <div className="absolute -top-2 left-1/2 -translate-x-1/2 bg-blue-500 text-[10px] px-2 py-0.5 rounded-full font-bold">
                                            #{leaf.index}
                                        </div>
                                        <p className="text-[10px] text-blue-300 font-mono mb-2 truncate">
                                            {leaf.hash}
                                        </p>
                                        <div className="bg-slate-950/50 rounded p-1.5 mt-2">
                                            <p className="text-xs text-white truncate font-bold">{leaf.key}</p>
                                            <p className="text-[10px] text-slate-400 truncate">{leaf.value}</p>
                                        </div>
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>

                    <div className="h-10 w-0.5 bg-gradient-to-b from-blue-500 to-emerald-500 opacity-50"></div>

                    {/* 2. 证明路径层：展示公共邻居节点 */}
                    <div className="w-full max-w-2xl">
                        <h3 className="text-center text-xs font-semibold text-emerald-400 uppercase tracking-widest mb-6">公共证明路径 (Proof Path)</h3>
                        <div className="space-y-3">
                            {proofData.proof.map((p: string, i: number) => (
                                <div key={i} className="flex flex-col items-center">
                                    <div className="bg-emerald-900/20 border border-emerald-500/30 px-6 py-3 rounded-lg w-full text-center group hover:bg-emerald-900/30 transition">
                                        <p className="text-[10px] text-emerald-500 mb-1">PROOF NODE {i + 1}</p>
                                        <p className="font-mono text-xs text-emerald-100 break-all">{p}</p>
                                    </div>
                                    {i !== proofData.proof.length - 1 && (
                                        <div className="h-4 w-px bg-emerald-500/20"></div>
                                    )}
                                </div>
                            ))}
                        </div>
                    </div>

                    <div className="h-10 w-0.5 bg-gradient-to-b from-emerald-500 to-purple-500 opacity-50"></div>

                    {/* 3. 根节点层：最终的权威结果 */}
                    <div className="w-full">
                        <h3 className="text-center text-xs font-semibold text-purple-400 uppercase tracking-widest mb-6">默克尔根 (Merkle Root)</h3>
                        <div className="flex justify-center">
                            <div className="bg-purple-900/40 border-2 border-purple-500 p-8 rounded-2xl text-center shadow-[0_0_30px_rgba(168,85,247,0.3)] max-w-3xl">
                                <p className="text-xs text-purple-300 mb-2 font-bold uppercase">Global Root Hash</p>
                                <p className="font-mono text-sm md:text-base text-white break-all leading-relaxed">
                                    {proofData.merkle_root}
                                </p>
                            </div>
                        </div>
                    </div>

                </div>
            </div>
        </div>
    );
}