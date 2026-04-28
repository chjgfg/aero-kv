
// "use client";

// import React, { useState, useEffect } from 'react';
// import { health, initAuth, setAuth, setPause, initFee, setFee, initStorage, upsert, deleted, scan, gets, page } from "../../utils/http.ts";

// export default function AdminDashboard() {
//     const [logs, setLogs] = useState<string[]>([]);
//     const [adminAddress, setAdminAddress] = useState("");
//     const [kvKey, setKvKey] = useState("");
//     const [kvValue, setKvValue] = useState("");
//     const [kvOffset, setKvOffset] = useState(0);
//     const [kvLimit, setKvLimit] = useState(10);
//     const [feeBase, setFeeBase] = useState(1000);
//     const [feePerByte, setFeePerByte] = useState(1);
//     const [feeScan, setFeeScan] = useState(10);

//     // 日志打印
//     const addLog = (msg: string) => {
//         setLogs(prev => [`[${new Date().toLocaleTimeString()}] ${msg}`, ...prev.slice(0, 19)]);
//     };

//     // 统一 API 调用
//     const handleAction = async (name: string, apiFunc: () => Promise<any>) => {
//         try {
//             addLog(`正在执行: ${name}...`);
//             const result = await apiFunc();
//             addLog(`✅ ${name} 成功: ${JSON.stringify(result)}`);
//             return true;
//         } catch (err: any) {
//             addLog(`❌ ${name} 失败: ${err.message}`);
//             return false;
//         }
//     };

//     // ==============================================
//     // 🔥 页面加载自动执行：initAuth → initFee → initStorage
//     // ==============================================
//     useEffect(() => {
//         const autoInit = async () => {
//             addLog("📦 页面加载完成，开始自动初始化...");

//             // 按顺序执行，必须前一个成功才执行下一个
//             const authOk = await handleAction("初始化权限", initAuth);
//             // if (!authOk) return;

//             const feeOk = await handleAction("初始化费用", initFee);
//             // if (!feeOk) return;

//             const storageOk = await handleAction("初始化存储", initStorage);
//             // if (storageOk) {
//             //     addLog("✅ 所有初始化任务执行完成！");
//             // }
//         };

//         // 进入页面立即执行一次
//         autoInit();
//     }, []); // 空依赖 = 只执行一次

//     return (
//         <div style={{ padding: '20px', maxWidth: '1200px', margin: '0 auto', fontFamily: 'sans-serif' }}>
//             <h1 style={{ borderBottom: '2px solid #333' }}>DiamondDB 管理控制台</h1>

//             <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
//                 <div>
//                     <section style={sectionStyle}>
//                         <h3>🚀 系统初始化 (依次执行)</h3>
//                         <div style={btnGroupStyle}>
//                             <button onClick={() => handleAction("健康检查", health)}>Health Check</button>
//                             <button onClick={() => handleAction("初始化权限", initAuth)} style={initBtnStyle}>Init Auth</button>
//                             <button onClick={() => handleAction("初始化费用", initFee)} style={initBtnStyle}>Init Fee</button>
//                             <button onClick={() => handleAction("初始化存储", initStorage)} style={initBtnStyle}>Init Storage</button>
//                         </div>
//                     </section>

//                     <section style={sectionStyle}>
//                         <h3>🔐 权限控制</h3>
//                         <input
//                             placeholder="新管理员钱包地址 (Base58)"
//                             value={adminAddress}
//                             onChange={(e) => setAdminAddress(e.target.value)}
//                             style={inputStyle}
//                         />
//                         <div style={btnGroupStyle}>
//                             <button onClick={() => handleAction("设置管理员", () => setAuth(adminAddress))}>更新管理员</button>
//                             <button onClick={() => handleAction("暂停合约", () => setPause(true))} style={{ backgroundColor: '#ff4d4f' }}>紧急暂停</button>
//                             <button onClick={() => handleAction("恢复合约", () => setPause(false))} style={{ backgroundColor: '#52c41a' }}>恢复运行</button>
//                         </div>
//                     </section>

//                     <section style={sectionStyle}>
//                         <h3>📦 KV 存储操作 (SkipList + 分页)</h3>
//                         <div style={{ display: 'flex', gap: '10px', marginBottom: '10px' }}>
//                             <input placeholder="Key" value={kvKey} onChange={(e) => setKvKey(e.target.value)} style={inputStyle} />
//                             <input placeholder="Value" value={kvValue} onChange={(e) => setKvValue(e.target.value)} style={inputStyle} />
//                         </div>
//                         {/* 分页：offset + limit */}
//                         <div style={{ display: 'flex', gap: '10px', marginBottom: '10px' }}>
//                             <input
//                                 type="number"
//                                 placeholder="Offset 起始位置"
//                                 value={kvOffset}
//                                 onChange={(e) => setKvOffset(Number(e.target.value))}
//                                 style={{ ...inputStyle, width: '120px' }}
//                             />
//                             <input
//                                 type="number"
//                                 placeholder="Limit 条数"
//                                 value={kvLimit}
//                                 onChange={(e) => setKvLimit(Number(e.target.value))}
//                                 style={{ ...inputStyle, width: '120px' }}
//                             />
//                         </div>
//                         <div style={btnGroupStyle}>
//                             <button onClick={() => handleAction("写入(Upsert)", () => upsert(kvKey, kvValue))}>写入数据</button>
//                             <button onClick={() => handleAction("查询(Get)", () => gets(kvKey))}>单点查询</button>
//                             <button onClick={() => handleAction("范围扫描(Scan)", () => scan(kvKey, kvOffset))}>范围扫描</button>
//                             <button onClick={() => handleAction("分页(Page)", () => page(kvOffset, kvLimit))}>分页</button>
//                             <button onClick={() => handleAction("删除(Delete)", () => deleted(kvKey))} style={{ backgroundColor: '#ff4d4f' }}>删除Key</button>
//                         </div>
//                     </section>

//                     <section style={sectionStyle}>
//                         <h3>💰 手续费配置</h3>
//                         <div style={{ display: 'flex', gap: '10px', marginBottom: '10px' }}>
//                             <label>基础: <input type="number" value={feeBase} onChange={(e) => setFeeBase(Number(e.target.value))} style={inputStyle} /></label>
//                             <label>字节: <input type="number" value={feePerByte} onChange={(e) => setFeePerByte(Number(e.target.value))} style={inputStyle} /></label>
//                             <label>扫描: <input type="number" value={feeScan} onChange={(e) => setFeeScan(Number(e.target.value))} style={inputStyle} /></label>
//                         </div>
//                         <button onClick={() => handleAction("更新费用", () => setFee(feeBase, feePerByte, feeScan))}>保存费用配置</button>
//                     </section>
//                 </div>

//                 <div>
//                     <section style={{ ...sectionStyle, height: '100%', backgroundColor: '#1e1e1e', color: '#00ff00' }}>
//                         <h3>📡 实时运行日志</h3>
//                         <div style={{ fontSize: '12px', lineHeight: '1.6', fontFamily: 'monospace' }}>
//                             {logs.map((log, i) => <div key={i} style={{ borderBottom: '1px solid #333', padding: '4px 0' }}>{log}</div>)}
//                         </div>
//                     </section>
//                 </div>
//             </div>
//         </div>
//     );
// }

// const sectionStyle: React.CSSProperties = {
//     border: '1px solid #ddd',
//     padding: '15px',
//     borderRadius: '8px',
//     marginBottom: '20px',
//     backgroundColor: '#f9f9f9'
// };

// const inputStyle: React.CSSProperties = {
//     padding: '8px',
//     borderRadius: '4px',
//     border: '1px solid #ccc',
//     width: '100%',
//     marginBottom: '10px',
//     color: '#333'
// };

// const btnGroupStyle: React.CSSProperties = {
//     display: 'flex',
//     gap: '10px',
//     flexWrap: 'wrap'
// };

// const initBtnStyle: React.CSSProperties = {
//     backgroundColor: '#1890ff',
//     color: 'white'
// };



"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import {
    health,
    initAuth,
    setPause,
    initFee,
    initStorage,
    upsert,
    deleted,
    gets,
    scan,
    page,
    initCounter,
} from "@/utils/http";
import { verifySingleProof } from "@/utils/merkle";

type TableItem = {
    key: string;
    value: string;
};

export default function KVAdminPage() {
    // 系统初始化
    const [initResult, setInitResult] = useState("");
    const [loadingInit, setLoadingInit] = useState(false);

    // 全局结果
    const [resJson, setResJson] = useState<any>(null);
    const [merkleStatus, setMerkleStatus] = useState("");

    // 表格数据
    const [tableList, setTableList] = useState<TableItem[]>([]);

    // 写入用
    const [writeKey, setWriteKey] = useState("");
    const [writeValue, setWriteValue] = useState("");

    // 查询用
    const [queryKey, setQueryKey] = useState("");

    // 范围扫描
    const [scanStartKey, setScanStartKey] = useState("");
    const [scanLimit, setScanLimit] = useState(10);

    // 分页
    const [pageNum, setPageNum] = useState(1);
    const [pageSize] = useState(10);

    // 删除
    const [delKey, setDelKey] = useState("");

    // 权限提示
    const [ctrlMsg, setCtrlMsg] = useState("");

    // 页面挂载自动加载分页列表
    useEffect(() => {
        fetchPageData();
    }, []);

    // 初始化系统
    const handleInitAll = async () => {
        setLoadingInit(true);
        setInitResult("");
        try {
            await health();
            setInitResult((p) => p + "✅ 健康检查完成\n");
            await initAuth();
            setInitResult((p) => p + "✅ 权限初始化完成\n");
            await initFee();
            setInitResult((p) => p + "✅ 手续费初始化完成\n");
            await initStorage();
            setInitResult((p) => p + "✅ 存储初始化完成\n");            
            await initCounter();
            setInitResult((p) => p + "✅ 总数据初始化完成\n");
        } catch (e: any) {
            setInitResult((p) => p + `❌ 初始化异常：${e.message}\n`);
        } finally {
            setLoadingInit(false);
        }
    };

    // 暂停 / 恢复
    const handlePause = async (paused: boolean) => {
        try {
            await setPause(paused);
            setCtrlMsg(paused ? "🔴 系统已暂停" : "🟢 系统已恢复");
        } catch {
            setCtrlMsg("❌ 操作失败");
        }
    };

    // 写入数据
    const handleUpsert = async () => {
        try {
            const res = await upsert(writeKey, writeValue);
            setResJson(res);
            fetchPageData();
            setWriteKey("");
            setWriteValue("");
        } catch (e: any) {
            setResJson({ error: e.message });
        }
    };

    // 单点查询
    const handleGetSingle = async () => {
        try {
            const res = await gets(queryKey);
            setResJson(res);
            const valid = await verifySingleProof(res);
            setMerkleStatus(valid ? "✅ Merkle 防篡改验证通过" : "❌ 数据篡改风险，验证失败");

            if (res?.key && res?.value) {
                setTableList([
                    {
                        key: res.key,
                        value: res.value,
                    },
                ]);
            }
        } catch (e: any) {
            setResJson({ error: e.message });
            setMerkleStatus("");
        }
    };

    // 范围扫描
    const handleScan = async () => {
        try {
            const res = await scan(scanStartKey, scanLimit);
            setResJson(res);
            if (Array.isArray(res?.pairs)) {
                setTableList(res.pairs);
            }
        } catch (e: any) {
            setResJson({ error: e.message });
        }
    };

    // 分页加载
    const fetchPageData = async () => {
        try {
            const res = await page(pageNum, pageSize);
            setResJson(res);
            if (Array.isArray(res?.pairs)) {
                setTableList(res.pairs);
            } else {
                setTableList([]);
            }
        } catch (e: any) {
            setResJson({ error: e.message });
            setTableList([]);
        }
    };

    // 删除
    const handleDelete = async () => {
        try {
            await deleted(delKey);
            fetchPageData();
            setDelKey("");
        } catch (e: any) {
            setResJson({ error: e.message });
        }
    };

    return (
        <div className="min-h-screen bg-slate-900 text-slate-100 p-6">
            <div className="max-w-5xl mx-auto space-y-6">
                <h1 className="text-3xl font-bold text-center text-sky-400">
                    KV 存储 + Merkle 防篡改管理平台
                </h1>

                {/* 1. 系统初始化 */}
                <div className="bg-slate-800 rounded-xl p-5 shadow">
                    <h2 className="text-lg font-semibold mb-3 text-sky-300">🚀 系统一键初始化</h2>
                    <button
                        onClick={handleInitAll}
                        disabled={loadingInit}
                        className="px-5 py-2 bg-sky-600 rounded-lg hover:bg-sky-500 disabled:opacity-60 transition"
                    >
                        {loadingInit ? "执行中..." : "开始初始化"}
                    </button>
                    <pre className="mt-3 p-3 bg-slate-950 rounded text-emerald-400 text-sm h-28 overflow-auto">
                        {initResult || "等待初始化操作..."}
                    </pre>
                </div>

                {/* 2. 系统控制 */}
                <div className="bg-slate-800 rounded-xl p-5 shadow">
                    <h2 className="text-lg font-semibold mb-3 text-amber-300">⚙️ 系统运行控制</h2>
                    <div className="flex gap-4">
                        <button
                            onClick={() => handlePause(true)}
                            className="px-5 py-2 bg-rose-600 rounded-lg hover:bg-rose-500 transition"
                        >
                            紧急暂停
                        </button>
                        <button
                            onClick={() => handlePause(false)}
                            className="px-5 py-2 bg-emerald-600 rounded-lg hover:bg-emerald-500 transition"
                        >
                            恢复运行
                        </button>
                    </div>
                    {ctrlMsg && <p className="mt-3 text-sm text-amber-300">{ctrlMsg}</p>}
                </div>

                {/* 3. 写入数据 */}
                <div className="bg-slate-800 rounded-xl p-5 shadow">
                    <h2 className="text-lg font-semibold mb-3 text-green-300">📝 新增 / 修改 KV 数据</h2>
                    <div className="grid grid-cols-2 gap-4 mb-3">
                        <input
                            value={writeKey}
                            onChange={(e) => setWriteKey(e.target.value)}
                            placeholder="请输入 Key"
                            className="bg-slate-700 rounded-lg px-4 py-2 outline-none focus:ring-2 focus:ring-green-500"
                        />
                        <input
                            value={writeValue}
                            onChange={(e) => setWriteValue(e.target.value)}
                            placeholder="请输入 Value"
                            className="bg-slate-700 rounded-lg px-4 py-2 outline-none focus:ring-2 focus:ring-green-500"
                        />
                    </div>
                    <button
                        onClick={handleUpsert}
                        className="px-5 py-2 bg-green-600 rounded-lg hover:bg-green-500 transition"
                    >
                        提交写入
                    </button>
                </div>

                {/* 4. 单点查询 */}
                <div className="bg-slate-800 rounded-xl p-5 shadow">
                    <h2 className="text-lg font-semibold mb-3 text-teal-300">🔍 单点精准查询</h2>
                    <div className="flex gap-4">
                        <input
                            value={queryKey}
                            onChange={(e) => setQueryKey(e.target.value)}
                            placeholder="输入需要查询的 Key"
                            className="flex-1 bg-slate-700 rounded-lg px-4 py-2 outline-none focus:ring-2 focus:ring-teal-500"
                        />
                        <button
                            onClick={handleGetSingle}
                            className="px-5 py-2 bg-teal-600 rounded-lg hover:bg-teal-500 transition"
                        >
                            查询
                        </button>
                    </div>
                    {merkleStatus && (
                        <p
                            className={`mt-3 text-sm ${merkleStatus.includes("通过") ? "text-emerald-400" : "text-rose-400"
                                }`}
                        >
                            {merkleStatus}
                        </p>
                    )}
                </div>

                {/* 5. 范围扫描 */}
                <div className="bg-slate-800 rounded-xl p-5 shadow">
                    <h2 className="text-lg font-semibold mb-3 text-violet-300">📃 范围扫描查询</h2>
                    <div className="grid grid-cols-2 gap-4 mb-3">
                        <input
                            value={scanStartKey}
                            onChange={(e) => setScanStartKey(e.target.value)}
                            placeholder="起始 Key"
                            className="bg-slate-700 rounded-lg px-4 py-2 outline-none focus:ring-2 focus:ring-violet-500"
                        />
                        <input
                            type="number"
                            value={scanLimit}
                            onChange={(e) => setScanLimit(Number(e.target.value))}
                            placeholder="获取条数"
                            className="bg-slate-700 rounded-lg px-4 py-2 outline-none focus:ring-2 focus:ring-violet-500"
                        />
                    </div>
                    <button
                        onClick={handleScan}
                        className="px-5 py-2 bg-violet-600 rounded-lg hover:bg-violet-500 transition"
                    >
                        开始扫描
                    </button>
                </div>

                {/* 6. 分页控制 */}
                <div className="bg-slate-800 rounded-xl p-5 shadow">
                    <h2 className="text-lg font-semibold mb-3 text-orange-300">📋 分页列表管理</h2>
                    <button
                        onClick={fetchPageData}
                        className="px-5 py-2 bg-orange-600 rounded-lg hover:bg-orange-500 transition"
                    >
                        刷新全表
                    </button>
                </div>

                {/* 7. 删除操作 */}
                <div className="bg-slate-800 rounded-xl p-5 shadow">
                    <h2 className="text-lg font-semibold mb-3 text-rose-300">🗑️ 删除指定 Key</h2>
                    <div className="flex gap-4">
                        <input
                            value={delKey}
                            onChange={(e) => setDelKey(e.target.value)}
                            placeholder="输入要删除的 Key"
                            className="flex-1 bg-slate-700 rounded-lg px-4 py-2 outline-none focus:ring-2 focus:ring-rose-500"
                        />
                        <button
                            onClick={handleDelete}
                            className="px-5 py-2 bg-rose-600 rounded-lg hover:bg-rose-500 transition"
                        >
                            删除
                        </button>
                    </div>
                </div>

                {/* 8. 表格数据展示 */}
                <div className="bg-slate-800 rounded-xl p-5 shadow">
                    <h2 className="text-lg font-semibold mb-4 text-slate-200">📊 数据列表表格</h2>
                    <div className="overflow-auto rounded-lg border border-slate-700">
                        <table className="w-full text-sm">
                            <thead className="bg-slate-700">
                                <tr>
                                    <th className="px-4 py-3 text-left w-1/3">Key</th>
                                    <th className="px-4 py-3 text-left">Value</th>
                                </tr>
                            </thead>
                            <tbody>
                                {tableList.length > 0 ? (
                                    tableList.map((item, idx) => (
                                        <tr key={idx} className="border-t border-slate-700 hover:bg-slate-700/50">
                                            <td className="px-4 py-3 font-mono">{item.key}</td>
                                            <td className="px-4 py-3 font-mono">{item.value}</td>
                                        </tr>
                                    ))
                                ) : (
                                    <tr>
                                        <td colSpan={2} className="px-4 py-6 text-center text-slate-400">
                                            暂无数据
                                        </td>
                                    </tr>
                                )}
                            </tbody>
                        </table>
                    </div>
                </div>

                {/* 9. 原始 JSON 结果 */}
                <div className="bg-slate-800 rounded-xl p-5 shadow">
                    <h2 className="text-lg font-semibold mb-3 text-slate-300">📤 原始接口返回</h2>
                    <pre className="p-4 bg-slate-950 rounded text-slate-300 text-xs max-h-60 overflow-auto">
                        {resJson ? JSON.stringify(resJson, null, 2) : "暂无返回数据"}
                    </pre>
                </div>

                <div className="text-center pt-2">
                    <Link href="/visual" className="text-sky-400 hover:underline">
                        前往 Merkle 树可视化页面 →
                    </Link>
                </div>
            </div>
        </div>
    );
}