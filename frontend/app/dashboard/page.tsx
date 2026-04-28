// "use client";

// import { useWallet } from "@solana/wallet-adapter-react";
// import { useEffect, useState } from "react";
// import { useRouter } from "next/navigation";

// type HealthResponse = {
//     status: string;
//     program_id: string;
//     message?: string;
// };

// export default function DashboardPage() {
//     const { connected, publicKey, disconnect } = useWallet();
//     const router = useRouter();
//     const [health, setHealth] = useState<HealthResponse | null>(null);
//     const [loading, setLoading] = useState(true);

//     // 登录守卫：未连接钱包自动跳回登录
//     useEffect(() => {
//         if (!connected) {
//             router.replace("/login");
//         }
//     }, [connected, router]);

//     // 拉取后端健康状态
//     useEffect(() => {
//         const fetchHealth = async () => {
//             if (!connected) return;
//             try {
//                 // 替换为你的后端地址
//                 const res = await fetch("http://192.168.40.131/health");
//                 const data = await res.json();
//                 console.log(data);
//                 setHealth(data);
//             } catch (err) {
//                 console.error("健康检查失败:", err);
//             } finally {
//                 setLoading(false);
//             }
//         };

//         fetchHealth();
//     }, [connected]);

//     // 加载中/未登录状态
//     if (!connected) return null;

//     return (
//         <div className="min-h-screen bg-[#0f172a] text-white">
//             {/* 顶部导航 */}
//             <header className="bg-[#1e293b] shadow-md px-6 py-4 flex justify-between items-center">
//                 <h1 className="text-xl font-bold text-blue-400">Aero KV 管理后台</h1>
//                 <div className="flex items-center gap-4">
//                     <p className="text-sm text-gray-300 font-mono">
//                         {publicKey?.toBase58().slice(0, 8)}...
//                     </p>
//                     <button
//                         onClick={disconnect}
//                         className="text-sm px-3 py-1 bg-red-500 hover:bg-red-600 rounded transition"
//                     >
//                         退出登录
//                     </button>
//                 </div>
//             </header>

//             {/* 主内容区 */}
//             <main className="max-w-7xl mx-auto px-4 py-8">
//                 {/* 服务状态卡片 */}
//                 <div className="bg-[#1e293b] p-6 rounded-2xl shadow mb-8">
//                     <h2 className="text-lg font-semibold mb-4">服务状态</h2>
//                     {loading ? (
//                         <p className="text-gray-400">加载中...</p>
//                     ) : health ? (
//                         <div className="space-y-2">
//                             <div className="flex items-center gap-2">
//                                 <span className="w-2 h-2 rounded-full bg-green-400"></span>
//                                 <span>状态：{health.status}</span>
//                             </div>
//                             <div className="text-sm text-gray-300">
//                                 程序ID：{health.program_id}
//                             </div>
//                         </div>
//                     ) : (
//                         <p className="text-red-400">后端服务连接失败</p>
//                     )}
//                 </div>

//                 {/* 功能入口卡片 */}
//                 <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
//                     <FeatureCard
//                         title="KV 数据管理"
//                         desc="新增、查询、修改、删除链上 KV 数据"
//                         path="/dashboard/kv"
//                     />
//                     <FeatureCard
//                         title="权限管理"
//                         desc="设置管理员、暂停/启动合约"
//                         path="/dashboard/auth"
//                     />
//                     <FeatureCard
//                         title="手续费管理"
//                         desc="初始化手续费、设置费率"
//                         path="/dashboard/fee"
//                     />
//                     <FeatureCard
//                         title="数据扫描"
//                         desc="扫描所有链上存储数据"
//                         path="/dashboard/scan"
//                     />
//                 </div>
//             </main>
//         </div>
//     );
// }

// // 功能卡片组件
// function FeatureCard({
//     title,
//     desc,
//     path,
// }: {
//     title: string;
//     desc: string;
//     path: string;
// }) {
//     const router = useRouter();
//     return (
//         <div
//             onClick={() => router.push(path)}
//             className="bg-[#1e293b] p-6 rounded-2xl shadow hover:bg-[#2a3756] cursor-pointer transition"
//         >
//             <h3 className="text-lg font-bold text-blue-400 mb-2">{title}</h3>
//             <p className="text-gray-300 text-sm">{desc}</p>
//         </div>
//     );
// }


"use client";

import { useState } from "react";
// import { putData, getData, scanData, deleteData } from "@/utils/http";
import { health, initAuth, setAuth, setPause, initFee, setFee, initStorage, upsert, deleted, scan, gets, page } from "../../utils/http.ts";
import { verifySingleProof } from "@/utils/merkle";
import Link from "next/link";

type ResultType = "idle" | "loading" | "success" | "error";

export default function KVPage() {
    const [putKey, setPutKey] = useState("");
    const [putValue, setPutValue] = useState("");
    const [getKey, setGetKey] = useState("");
    const [scanKey, setScanKey] = useState("");
    const [scanLimit, setScanLimit] = useState(5);
    const [delKey, setDelKey] = useState("");

    const [result, setResult] = useState<any>(null);
    const [resultType, setResultType] = useState<ResultType>("idle");
    const [merkleValid, setMerkleValid] = useState<boolean | null>(null);
    const [lastProof, setLastProof] = useState<any>(null);

    const showResult = (data: any, valid: boolean | null) => {
        setResult(data);
        setResultType("success");
        setMerkleValid(valid);
        if (data?.proof) {
            setLastProof({
                proof: data.proof,
                merkle_root: data.merkle_root,
                key_hash: data.key_hash,
                leaf_index: data.leaf_index,
            });
            localStorage.setItem("lastProof", JSON.stringify(lastProof));
        }
    };

    const handlePut = async () => {
        setResultType("loading");
        try {
            const data = await upsert(putKey, putValue);
            showResult(data, null);
        } catch (err) {
            setResultType("error");
        }
    };

    const handleGet = async () => {
        setResultType("loading");
        try {
            const data = await gets(getKey);
            const valid = await verifySingleProof(data);
            showResult(data, valid);
        } catch (err) {
            setResultType("error");
        }
    };

    const handleScan = async () => {
        setResultType("loading");
        try {
            const data = await scan(scanKey, scanLimit);
            showResult(data, null);
        } catch (err) {
            setResultType("error");
        }
    };

    const handleDelete = async () => {
        setResultType("loading");
        try {
            const data = await deleted(delKey);
            showResult(data, null);
        } catch (err) {
            setResultType("error");
        }
    };

    return (
        <div className="min-h-screen bg-gray-900 text-gray-100 p-8">
            <div className="max-w-4xl mx-auto">
                <div className="flex justify-between items-center mb-8">
                    <h1 className="text-3xl font-bold text-blue-400">🔐 KV + Merkle 安全存储</h1>
                    <Link href="/visual" className="px-4 py-2 bg-purple-600 rounded-lg hover:bg-purple-500 transition">
                        📊 查看 Merkle 可视化
                    </Link>
                </div>

                <div className="grid gap-6">
                    {/* 写入 */}
                    <div className="bg-gray-800 rounded-xl p-6 shadow-lg">
                        <h2 className="text-xl font-semibold mb-4 text-blue-300">📝 写入数据 (PUT)</h2>
                        <div className="flex gap-4 mb-4">
                            <input
                                type="text"
                                placeholder="Key"
                                value={putKey}
                                onChange={(e) => setPutKey(e.target.value)}
                                className="flex-1 bg-gray-700 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            />
                            <input
                                type="text"
                                placeholder="Value"
                                value={putValue}
                                onChange={(e) => setPutValue(e.target.value)}
                                className="flex-1 bg-gray-700 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            />
                        </div>
                        <button onClick={handlePut} className="px-6 py-2 bg-blue-600 rounded-lg hover:bg-blue-500 transition">提交保存</button>
                    </div>

                    {/* 查询 */}
                    <div className="bg-gray-800 rounded-xl p-6 shadow-lg">
                        <h2 className="text-xl font-semibold mb-4 text-green-300">🔍 查询数据 (GET)</h2>
                        <div className="flex gap-4 mb-4">
                            <input
                                type="text"
                                placeholder="输入 Key 查询"
                                value={getKey}
                                onChange={(e) => setGetKey(e.target.value)}
                                className="flex-1 bg-gray-700 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-green-500"
                            />
                        </div>
                        <button onClick={handleGet} className="px-6 py-2 bg-green-600 rounded-lg hover:bg-green-500 transition">查询</button>
                        {merkleValid !== null && (
                            <div className={`mt-4 p-4 rounded-lg ${merkleValid ? "bg-green-900/30 border border-green-500" : "bg-red-900/30 border border-red-500"}`}>
                                <p className="font-semibold">Merkle 验证结果: {merkleValid ? "✅ 通过（数据未被篡改）" : "❌ 失败（数据被篡改）"}</p>
                            </div>
                        )}
                    </div>

                    {/* 结果展示 */}
                    <div className="bg-gray-800 rounded-xl p-6 shadow-lg">
                        <h2 className="text-xl font-semibold mb-4 text-yellow-300">📋 响应结果</h2>
                        <pre className="bg-gray-900 p-4 rounded-lg overflow-auto max-h-64 text-sm text-gray-300">
                            {resultType === "loading" ? "请求中..." : JSON.stringify(result, null, 2) || "暂无数据"}
                        </pre>
                    </div>
                </div>
            </div>
        </div>
    );
}