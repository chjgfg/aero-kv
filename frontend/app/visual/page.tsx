"use client";

import { useEffect, useState } from "react";
import Link from "next/link";

export default function VisualPage() {
    const [proofData, setProofData] = useState<any>(null);

    useEffect(() => {
        const data = localStorage.getItem("lastProof");
        if (data) setProofData(JSON.parse(data));
    }, []);

    if (!proofData) {
        return (
            <div className="min-h-screen bg-gray-900 text-gray-100 flex flex-col items-center justify-center">
                <h1 className="text-3xl font-bold text-purple-400 mb-4">🌳 Merkle 可视化</h1>
                <p className="text-gray-400 mb-8">请先在主页查询一次数据，生成验证路径</p>
                <Link href="/" className="px-6 py-2 bg-purple-600 rounded-lg hover:bg-purple-500 transition">返回主页</Link>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-gray-900 text-gray-100 p-8">
            <div className="max-w-4xl mx-auto">
                <div className="flex justify-between items-center mb-8">
                    <h1 className="text-3xl font-bold text-purple-400">🌳 Merkle 验证路径</h1>
                    <Link href="/" className="px-4 py-2 bg-gray-700 rounded-lg hover:bg-gray-600 transition">返回主页</Link>
                </div>

                <div className="space-y-6">
                    {/* 叶子节点 */}
                    <div className="flex justify-center">
                        <div className="bg-blue-600 px-6 py-4 rounded-lg text-center">
                            <p className="text-sm opacity-80">Leaf Node</p>
                            <p className="font-mono text-sm">{proofData.key_hash.slice(0, 16)}...</p>
                        </div>
                    </div>

                    {/* 证明路径 */}
                    {proofData.proof.map((p: string, i: number) => (
                        <div key={i} className="flex justify-center">
                            <div className="bg-green-600 px-6 py-4 rounded-lg text-center">
                                <p className="text-sm opacity-80">Proof Node {i + 1}</p>
                                <p className="font-mono text-sm">{p.slice(0, 16)}...</p>
                            </div>
                        </div>
                    ))}

                    {/* 根节点 */}
                    <div className="flex justify-center">
                        <div className="bg-purple-600 px-6 py-4 rounded-lg text-center">
                            <p className="text-sm opacity-80">Root Node</p>
                            <p className="font-mono text-sm">{proofData.merkle_root.slice(0, 20)}...</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}