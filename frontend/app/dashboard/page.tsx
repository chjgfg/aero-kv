// "use client";

// import { useState, useEffect } from "react";
// import Link from "next/link";
// import {
//     health,
//     initAuth,
//     setPause,
//     initFee,
//     initStorage,
//     upsert,
//     deleted,
//     gets,
//     scan,
//     page,
//     initCounter,
// } from "@/utils/http";
// import { saveProofData, verifySingleProof } from "@/utils/merkle";
// import { mock } from "@/utils/mock";
// import { useRouter } from "next/navigation"; // 1. 确保导入了 useRouter

// type TableItem = {
//     key: string;
//     value: string;
// };

// export default function KVAdminPage() {
//     const router = useRouter(); // 2. 在组件内部初始化
//     // 系统初始化
//     const [initResult, setInitResult] = useState("");
//     const [loadingInit, setLoadingInit] = useState(false);

//     // 全局结果
//     const [resJson, setResJson] = useState<any>(null);
//     const [merkleStatus, setMerkleStatus] = useState("");

//     // 表格数据
//     const [tableList, setTableList] = useState<TableItem[]>([]);

//     // 写入用
//     const [writeKey, setWriteKey] = useState("");
//     const [writeValue, setWriteValue] = useState("");

//     // 查询用
//     const [queryKey, setQueryKey] = useState("");

//     // 范围扫描
//     const [scanStartKey, setScanStartKey] = useState("");
//     const [scanLimit, setScanLimit] = useState(10);
//     // 范围扫描：前端假分页专用
//     const [scanAllData, setScanAllData] = useState<TableItem[]>([]);
//     const [scanPageNum, setScanPageNum] = useState(1);
//     const [scanTotalPages, setScanTotalPages] = useState(1);
//     const [isScanMode, setIsScanMode] = useState(false);

//     // 普通分页
//     const [pageNum, setPageNum] = useState(1);
//     const [pageSize] = useState(10);
//     const [total, setTotal] = useState(0);
//     const [totalPages, setTotalPages] = useState(1);

//     // 删除
//     const [delKey, setDelKey] = useState("");

//     // 权限提示
//     const [ctrlMsg, setCtrlMsg] = useState("");

//     // ✅ 新增：单点查询模式标记
//     const [isSingleMode, setIsSingleMode] = useState(false);

//     // 页面挂载自动加载分页列表
//     useEffect(() => {
//         if (!isScanMode && !isSingleMode) {
//             fetchPageData();
//         }
//     }, [pageNum, isScanMode, isSingleMode]);

//     // 初始化系统
//     const handleInitAll = async () => {
//         setLoadingInit(true);
//         setInitResult("");
//         try {
//             await health();
//             setInitResult((p) => p + "✅ 健康检查完成\n");
//             await initAuth();
//             setInitResult((p) => p + "✅ 权限初始化完成\n");
//             await initFee();
//             setInitResult((p) => p + "✅ 手续费初始化完成\n");
//             await initStorage();
//             setInitResult((p) => p + "✅ 存储初始化完成\n");
//             await initCounter();
//             setInitResult((p) => p + "✅ 总数据初始化完成\n");
//         } catch (e: any) {
//             setInitResult((p) => p + `❌ 初始化异常：${e.message}\n`);
//         } finally {
//             setLoadingInit(false);
//         }
//     };

//     // 暂停 / 恢复
//     const handlePause = async (paused: boolean) => {
//         try {
//             await setPause(paused);
//             setCtrlMsg(paused ? "🔴 系统已暂停" : "🟢 系统已恢复");
//         } catch {
//             setCtrlMsg("❌ 操作失败");
//         }
//     };

//     // 写入数据
//     const handleUpsert = async () => {
//         try {
//             const res = await upsert(writeKey, writeValue);
//             setResJson(res);
//             // 写入后切回普通分页模式
//             setIsScanMode(false);
//             setIsSingleMode(false);
//             setPageNum(1);
//             fetchPageData();
//             setWriteKey("");
//             setWriteValue("");
//         } catch (e: any) {
//             setResJson({ error: e.message });
//         }
//     };

//     // ✅ 单点查询：新增 isSingleMode 标记
//     const handleGetSingle = async () => {
//         try {
//             const res = await gets(queryKey);
//             setResJson(res);
//             const valid = await verifySingleProof(res);
//             setMerkleStatus(valid ? "✅ Merkle 防篡改验证通过" : "❌ 数据篡改风险，验证失败");

//             if (res?.key && res?.value) {
//                 setTableList([{ key: res.key, value: res.value }]);
//                 saveProofData([res], [res.key_hash], [res.leaf_index], res.proof, res.merkle_root);
//                 // 标记为单点查询模式，隐藏分页
//                 setIsSingleMode(true);
//                 setIsScanMode(false);
//             }
//         } catch (e: any) {
//             setResJson({ error: e.message });
//             setMerkleStatus("");
//         }
//     };

//     // 范围扫描：前端假分页（不修改后端接口）
//     const handleScan = async () => {
//         try {
//             // 1. 调用后端接口，一次性拿完所有数据
//             const res = await scan(scanStartKey, scanLimit);
//             setResJson(res);

//             if (Array.isArray(res?.pairs)) {
//                 saveProofData(res.pairs, res.key_hashes, res.leaf_indices, res.proof, res.merkle_root);
//                 // 2. 保存全部数据，用于前端假分页
//                 setScanAllData(res.pairs);
//                 setScanPageNum(1);
//                 setScanTotalPages(Math.ceil(res.pairs.length / pageSize));

//                 // 3. 初始显示第一页
//                 setTableList(res.pairs.slice(0, pageSize));

//                 // 4. 标记为扫描模式
//                 setIsScanMode(true);
//                 setIsSingleMode(false);
//             }
//         } catch (e: any) {
//             setResJson({ error: e.message });
//         }
//     };

//     // ✅ 前端假分页跳转（扫描模式专用）
//     const goToScanPage = (num: number) => {
//         if (num < 1 || num > scanTotalPages) return;
//         setScanPageNum(num);

//         const start = (num - 1) * pageSize;
//         const end = start + pageSize;
//         setTableList(scanAllData.slice(start, end));
//     };

//     // 普通分页加载
//     const fetchPageData = async () => {
//         try {
//             const res = await page(pageNum, pageSize);
//             console.log(res);
//             setResJson(res);

//             const totalItems = res.total || 0;
//             setTotal(totalItems);
//             setTotalPages(Math.ceil(totalItems / pageSize));

//             if (Array.isArray(res?.pairs)) {
//                 saveProofData(res.pairs, res.key_hashes, res.leaf_indices, res.proof, res.merkle_root);
//                 setTableList(res.pairs);
//             } else {
//                 setTableList([]);
//             }
//             // 切回普通模式
//             setIsScanMode(false);
//             setIsSingleMode(false);
//         } catch (e: any) {
//             setResJson({ error: e.message });
//             setTableList([]);
//         }
//     };

//     // 删除
//     const handleDelete = async () => {
//         try {
//             await deleted(delKey);
//             // 删除后切回普通分页模式
//             setIsScanMode(false);
//             setIsSingleMode(false);
//             setPageNum(1);
//             fetchPageData();
//             setDelKey("");
//         } catch (e: any) {
//             setResJson({ error: e.message });
//         }
//     };

//     // 普通分页跳转
//     const goToPage = (num: number) => {
//         if (num < 1 || num > totalPages) return;
//         setPageNum(num);
//     };

//     // ✅ 通用分页渲染：新增 isSingleMode 判断
//     const renderPagination = () => {
//         // 没有数据 或 单点查询模式：不显示分页
//         if (tableList.length === 0 || isSingleMode) return null;

//         if (isScanMode) {
//             // 扫描模式：前端假分页
//             const pages = [];
//             const maxVisible = 5;
//             let start = Math.max(1, scanPageNum - 2);
//             let end = Math.min(scanTotalPages, scanPageNum + 2);

//             for (let i = start; i <= end; i++) {
//                 pages.push(i);
//             }

//             return (
//                 <div className="flex items-center justify-between flex-wrap gap-4 mt-6 pt-4 border-t border-slate-700">
//                     <div className="flex items-center gap-2">
//                         <button
//                             onClick={() => goToScanPage(1)}
//                             disabled={scanPageNum === 1}
//                             className="px-3 py-1.5 bg-slate-700 rounded disabled:opacity-50 hover:bg-slate-600 transition text-sm"
//                         >
//                             首页
//                         </button>
//                         <button
//                             onClick={() => goToScanPage(scanPageNum - 1)}
//                             disabled={scanPageNum === 1}
//                             className="px-3 py-1.5 bg-slate-700 rounded disabled:opacity-50 hover:bg-slate-600 transition text-sm"
//                         >
//                             上一页
//                         </button>
//                         {pages.map((i) => (
//                             <button
//                                 key={i}
//                                 onClick={() => goToScanPage(i)}
//                                 className={`px-3 py-1.5 rounded transition text-sm ${scanPageNum === i ? "bg-violet-600 text-white" : "bg-slate-700 hover:bg-slate-600"
//                                     }`}
//                             >
//                                 {i}
//                             </button>
//                         ))}
//                         <button
//                             onClick={() => goToScanPage(scanPageNum + 1)}
//                             disabled={scanPageNum >= scanTotalPages}
//                             className="px-3 py-1.5 bg-slate-700 rounded disabled:opacity-50 hover:bg-slate-600 transition text-sm"
//                         >
//                             下一页
//                         </button>
//                         <button
//                             onClick={() => goToScanPage(scanTotalPages)}
//                             disabled={scanPageNum >= scanTotalPages}
//                             className="px-3 py-1.5 bg-slate-700 rounded disabled:opacity-50 hover:bg-slate-600 transition text-sm"
//                         >
//                             尾页
//                         </button>
//                     </div>
//                     <span className="text-sm text-slate-400">
//                         第 {scanPageNum} 页 / 共 {scanTotalPages} 页 | 总计 {scanAllData.length} 条
//                     </span>
//                 </div>
//             );
//         } else {
//             // 普通分页模式：后端分页
//             const pages = [];
//             const maxVisible = 5;
//             let start = Math.max(1, pageNum - 2);
//             let end = Math.min(totalPages, pageNum + 2);

//             for (let i = start; i <= end; i++) {
//                 pages.push(i);
//             }

//             return (
//                 <div className="flex items-center justify-between flex-wrap gap-4 mt-6 pt-4 border-t border-slate-700">
//                     <div className="flex items-center gap-2">
//                         <button
//                             onClick={() => goToPage(1)}
//                             disabled={pageNum === 1}
//                             className="px-3 py-1.5 bg-slate-700 rounded disabled:opacity-50 hover:bg-slate-600 transition text-sm"
//                         >
//                             首页
//                         </button>
//                         <button
//                             onClick={() => goToPage(pageNum - 1)}
//                             disabled={pageNum === 1}
//                             className="px-3 py-1.5 bg-slate-700 rounded disabled:opacity-50 hover:bg-slate-600 transition text-sm"
//                         >
//                             上一页
//                         </button>
//                         {pages.map((i) => (
//                             <button
//                                 key={i}
//                                 onClick={() => goToPage(i)}
//                                 className={`px-3 py-1.5 rounded transition text-sm ${pageNum === i ? "bg-sky-600 text-white" : "bg-slate-700 hover:bg-slate-600"
//                                     }`}
//                             >
//                                 {i}
//                             </button>
//                         ))}
//                         <button
//                             onClick={() => goToPage(pageNum + 1)}
//                             disabled={pageNum >= totalPages}
//                             className="px-3 py-1.5 bg-slate-700 rounded disabled:opacity-50 hover:bg-slate-600 transition text-sm"
//                         >
//                             下一页
//                         </button>
//                         <button
//                             onClick={() => goToPage(totalPages)}
//                             disabled={pageNum >= totalPages}
//                             className="px-3 py-1.5 bg-slate-700 rounded disabled:opacity-50 hover:bg-slate-600 transition text-sm"
//                         >
//                             尾页
//                         </button>
//                     </div>
//                     <span className="text-sm text-slate-400">
//                         第 {pageNum} 页 / 共 {totalPages} 页 | 总计 {total} 条
//                     </span>
//                 </div>
//             );
//         }
//     };

//     const mokeTest = async () => {
//         await moke();
//     }

//     return (
//         <div className="min-h-screen bg-slate-900 text-slate-100 p-6">
//             <div className="max-w-6xl mx-auto space-y-6">
//                 <h1 className="text-3xl font-bold text-center text-sky-400">
//                     AeroKV 存储
//                 </h1>

//                 {/* 顶部控制区 */}
//                 <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
//                     {/* 初始化 & 控制 */}
//                     <div className="bg-slate-800 rounded-xl p-5 shadow-lg">
//                         <h2 className="text-lg font-semibold mb-4 text-sky-300">🚀 系统控制</h2>
//                         <div className="space-y-4">
//                             <button
//                                 onClick={handleInitAll}
//                                 disabled={loadingInit}
//                                 className="w-full py-2 bg-sky-600 rounded-lg hover:bg-sky-500 disabled:opacity-60 transition font-medium"
//                             >
//                                 {loadingInit ? "执行中..." : "系统一键初始化"}
//                             </button>
//                             <div className="flex gap-3">
//                                 <button
//                                     onClick={() => handlePause(true)}
//                                     className="flex-1 py-2 bg-rose-600 rounded-lg hover:bg-rose-500 transition font-medium"
//                                 >
//                                     紧急暂停
//                                 </button>
//                                 <button
//                                     onClick={() => handlePause(false)}
//                                     className="flex-1 py-2 bg-emerald-600 rounded-lg hover:bg-emerald-500 transition font-medium"
//                                 >
//                                     恢复运行
//                                 </button>
//                             </div>
//                             {ctrlMsg && <p className="text-sm text-amber-300">{ctrlMsg}</p>}
//                             <pre className="p-3 bg-slate-950 rounded text-emerald-400 text-sm h-31 overflow-auto">
//                                 {initResult || "等待初始化操作..."}
//                             </pre>
//                         </div>
//                     </div>

//                     {/* 写入 & 删除 */}
//                     <div className="bg-slate-800 rounded-xl p-5 shadow-lg">
//                         <h2 className="text-lg font-semibold mb-4 text-green-300">📝 数据管理</h2>
//                         <div className="space-y-4">
//                             <div className="grid grid-cols-2 gap-3">
//                                 <input
//                                     value={writeKey}
//                                     onChange={(e) => setWriteKey(e.target.value)}
//                                     placeholder="Key"
//                                     className="bg-slate-700 rounded-lg px-4 py-2 outline-none focus:ring-2 focus:ring-green-500"
//                                 />
//                                 <input
//                                     value={writeValue}
//                                     onChange={(e) => setWriteValue(e.target.value)}
//                                     placeholder="Value"
//                                     className="bg-slate-700 rounded-lg px-4 py-2 outline-none focus:ring-2 focus:ring-green-500"
//                                 />
//                             </div>
//                             <button
//                                 onClick={handleUpsert}
//                                 className="w-full py-2 bg-green-600 rounded-lg hover:bg-green-500 transition font-medium"
//                             >
//                                 提交写入
//                             </button>
//                             <div className="flex gap-3">
//                                 <input
//                                     value={delKey}
//                                     onChange={(e) => setDelKey(e.target.value)}
//                                     placeholder="删除 Key"
//                                     className="flex-1 bg-slate-700 rounded-lg px-4 py-2 outline-none focus:ring-2 focus:ring-rose-500"
//                                 />
//                                 <button
//                                     onClick={handleDelete}
//                                     className="px-6 py-2 bg-rose-600 rounded-lg hover:bg-rose-500 transition font-medium"
//                                 >
//                                     删除
//                                 </button>
//                             </div>
//                             <div className="flex gap-3">
//                                 <button
//                                     onClick={mokeTest}
//                                     className="w-full py-2 bg-rose-600 rounded-lg hover:bg-rose-500 transition font-medium"
//                                 >
//                                     自动添加数据测试
//                                 </button>
//                             </div>
//                         </div>
//                     </div>
//                 </div>

//                 {/* 查询 & 扫描 */}
//                 <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
//                     <div className="bg-slate-800 rounded-xl p-5 shadow-lg">
//                         <h2 className="text-lg font-semibold mb-4 text-teal-300">🔍 单点查询</h2>
//                         <div className="flex gap-3">
//                             <input
//                                 value={queryKey}
//                                 onChange={(e) => setQueryKey(e.target.value)}
//                                 placeholder="输入 Key"
//                                 className="flex-1 bg-slate-700 rounded-lg px-4 py-2 outline-none focus:ring-2 focus:ring-teal-500"
//                             />
//                             <button
//                                 onClick={handleGetSingle}
//                                 className="px-6 py-2 bg-teal-600 rounded-lg hover:bg-teal-500 transition font-medium"
//                             >
//                                 查询
//                             </button>
//                         </div>
//                         {merkleStatus && (
//                             <p className={`mt-3 text-sm ${merkleStatus.includes("通过") ? "text-emerald-400" : "text-rose-400"}`}>
//                                 {merkleStatus}
//                             </p>
//                         )}
//                     </div>
//                     <div className="bg-slate-800 rounded-xl p-5 shadow-lg">
//                         <h2 className="text-lg font-semibold mb-4 text-violet-300">📃 范围扫描</h2>
//                         <div className="grid grid-cols-2 gap-3">
//                             <input
//                                 value={scanStartKey}
//                                 onChange={(e) => setScanStartKey(e.target.value)}
//                                 placeholder="起始 Key"
//                                 className="bg-slate-700 rounded-lg px-4 py-2 outline-none focus:ring-2 focus:ring-violet-500"
//                             />
//                             <input
//                                 type="number"
//                                 value={scanLimit}
//                                 onChange={(e) => setScanLimit(Number(e.target.value))}
//                                 placeholder="条数"
//                                 className="bg-slate-700 rounded-lg px-4 py-2 outline-none focus:ring-2 focus:ring-violet-500"
//                             />
//                         </div>
//                         <button
//                             onClick={handleScan}
//                             className="w-full mt-3 py-2 bg-violet-600 rounded-lg hover:bg-violet-500 transition font-medium"
//                         >
//                             开始扫描
//                         </button>
//                     </div>
//                 </div>

//                 {/* 主数据表格（最醒目位置） */}
//                 <div className="bg-slate-800 rounded-xl p-6 shadow-lg">
//                     <div className="flex justify-between items-center mb-4">
//                         <h2 className="text-xl font-semibold text-slate-200">
//                             {isScanMode ? "📊 扫描结果列表" : "📊 数据列表"}
//                         </h2>
//                         {/* 🌟 关键：用一个 div 把两个按钮包起来，并设置 flex 和 gap */}
//                         <div className="flex gap-3"> 
//                             <button
//                                 onClick={() => {
//                                     router.push("/visual");
//                                 }}
//                                 className="px-4 py-2 bg-orange-600 rounded-lg hover:bg-orange-500 transition text-sm font-medium"
//                             >
//                                 Merkle 可视化
//                             </button>
//                             <button
//                                 onClick={() => {
//                                     setIsScanMode(false);
//                                     setIsSingleMode(false);
//                                     setPageNum(1);
//                                     fetchPageData();
//                                 }}
//                                 className="px-4 py-2 bg-orange-600 rounded-lg hover:bg-orange-500 transition text-sm font-medium"
//                             >
//                                 刷新全表
//                             </button>
//                         </div>
//                     </div>
//                     <div className="overflow-auto rounded-lg border border-slate-700">
//                         <table className="w-full text-sm">
//                             <thead className="bg-slate-700">
//                                 <tr>
//                                     <th className="px-4 py-3 text-left w-1/3 font-semibold">Key</th>
//                                     <th className="px-4 py-3 text-left font-semibold">Value</th>
//                                     <th className="px-4 py-3 text-left font-semibold">Merkle</th>
//                                 </tr>
//                             </thead>
//                             <tbody>
//                                 {tableList.length > 0 ? (
//                                     tableList.map((item, idx) => (
//                                         <tr key={idx} className="border-t border-slate-700 hover:bg-slate-700/50 transition-colors">
//                                             <td className="px-4 py-3 font-mono">{item.key}</td>
//                                             <td className="px-4 py-3 font-mono">{item.value}</td>
//                                             <td className="px-4 py-3">
//                                                 <button 
//                                                     onClick={() => {
//                                                         // 从当前页面的 resJson 中提取对应的证据数据
//                                                         // 注意：page/scan 接口返回的数据在 resJson 里，我们需要根据 idx 取出对应的 hash
//                                                         const singleItemHash = resJson.key_hashes[idx];
//                                                         const singleItemIndex = resJson.leaf_indices[idx];
//                                                         saveProofData([item], [singleItemHash], [singleItemIndex], resJson.proof, resJson.merkle_root);
//                                                         router.push("/visual");
//                                                     }}
//                                                     className="text-sky-400 text-xs hover:underline"
//                                                 >
//                                                     验证此条
//                                                 </button>
//                                             </td>
//                                         </tr>
//                                     ))
//                                 ) : (
//                                     <tr>
//                                         <td colSpan={2} className="px-4 py-10 text-center text-slate-400">
//                                             暂无数据
//                                         </td>
//                                     </tr>
//                                 )}
//                             </tbody>
//                         </table>
//                     </div>

//                     {/* 通用分页条：现在会根据 isSingleMode 自动隐藏 */}
//                     {renderPagination()}
//                 </div>

//                 {/* 原始 JSON 结果（醒目位置） */}
//                 {/* <div className="bg-slate-800 rounded-xl p-6 shadow-lg">
//                     <h2 className="text-xl font-semibold mb-4 text-slate-300">📤 接口返回数据</h2>
//                     <pre className="p-4 bg-slate-950 rounded text-slate-300 text-sm max-h-72 overflow-auto">
//                         {resJson ? JSON.stringify(resJson, null, 2) : "暂无返回数据"}
//                     </pre>
//                 </div>

//                 <div className="text-center pt-2">
//                     <Link href="/visual" className="text-sky-400 hover:underline">
//                         前往 Merkle 树可视化页面 →
//                     </Link>
//                 </div> */}
//             </div>
//         </div>
//     );
// }

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
import { saveProofData, verifySingleProof } from "@/utils/merkle";
import { mock } from "@/utils/mock";
import { useRouter } from "next/navigation";

type TableItem = {
    key: string;
    value: string;
};

export default function KVAdminPage() {
    const router = useRouter();
    
    // --- 状态逻辑 (完全不动) ---
    const [initResult, setInitResult] = useState("");
    const [loadingInit, setLoadingInit] = useState(false);
    const [resJson, setResJson] = useState<any>(null);
    const [merkleStatus, setMerkleStatus] = useState("");
    const [tableList, setTableList] = useState<TableItem[]>([]);
    const [writeKey, setWriteKey] = useState("");
    const [writeValue, setWriteValue] = useState("");
    const [queryKey, setQueryKey] = useState("");
    const [scanStartKey, setScanStartKey] = useState("");
    const [scanLimit, setScanLimit] = useState(10);
    const [scanAllData, setScanAllData] = useState<TableItem[]>([]);
    const [scanPageNum, setScanPageNum] = useState(1);
    const [scanTotalPages, setScanTotalPages] = useState(1);
    const [isScanMode, setIsScanMode] = useState(false);
    const [pageNum, setPageNum] = useState(1);
    const [pageSize] = useState(10);
    const [total, setTotal] = useState(0);
    const [totalPages, setTotalPages] = useState(1);
    const [delKey, setDelKey] = useState("");
    const [ctrlMsg, setCtrlMsg] = useState("");
    const [isSingleMode, setIsSingleMode] = useState(false);

    useEffect(() => {
        if (!isScanMode && !isSingleMode) {
            fetchPageData();
        }
    }, [pageNum, isScanMode, isSingleMode]);

    // --- 方法逻辑 (完全不动) ---
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

    const handlePause = async (paused: boolean) => {
        try {
            await setPause(paused);
            setCtrlMsg(paused ? "🔴 系统已暂停" : "🟢 系统已恢复");
        } catch {
            setCtrlMsg("❌ 操作失败");
        }
    };

    const handleUpsert = async () => {
        try {
            const res = await upsert(writeKey, writeValue);
            setResJson(res);
            setIsScanMode(false);
            setIsSingleMode(false);
            setPageNum(1);
            fetchPageData();
            setWriteKey("");
            setWriteValue("");
        } catch (e: any) {
            setResJson({ error: e.message });
        }
    };

    const handleGetSingle = async () => {
        try {
            const res = await gets(queryKey);
            setResJson(res);
            const valid = await verifySingleProof(res);
            setMerkleStatus(valid ? "✅ Merkle 防篡改验证通过" : "❌ 验证失败");
            if (res?.key && res?.value) {
                setTableList([{ key: res.key, value: res.value }]);
                saveProofData([res], [res.key_hash], [res.leaf_index], res.proof, res.merkle_root);
                setIsSingleMode(true);
                setIsScanMode(false);
            }
        } catch (e: any) {
            setResJson({ error: e.message });
            setMerkleStatus("");
        }
    };

    const handleScan = async () => {
        try {
            const res = await scan(scanStartKey, scanLimit);
            setResJson(res);
            if (Array.isArray(res?.pairs)) {
                saveProofData(res.pairs, res.key_hashes, res.leaf_indices, res.proof, res.merkle_root);
                setScanAllData(res.pairs);
                setScanPageNum(1);
                setScanTotalPages(Math.ceil(res.pairs.length / pageSize));
                setTableList(res.pairs.slice(0, pageSize));
                setIsScanMode(true);
                setIsSingleMode(false);
            }
        } catch (e: any) {
            setResJson({ error: e.message });
        }
    };

    const fetchPageData = async () => {
        try {
            const res = await page(pageNum, pageSize);
            setResJson(res);
            const totalItems = res.total || 0;
            setTotal(totalItems);
            setTotalPages(Math.ceil(totalItems / pageSize));
            if (Array.isArray(res?.pairs)) {
                saveProofData(res.pairs, res.key_hashes, res.leaf_indices, res.proof, res.merkle_root);
                setTableList(res.pairs);
            } else { setTableList([]); }
            setIsScanMode(false);
            setIsSingleMode(false);
        } catch (e: any) {
            setResJson({ error: e.message });
            setTableList([]);
        }
    };

    const handleDelete = async () => {
        try {
            await deleted(delKey);
            setIsScanMode(false);
            setIsSingleMode(false);
            setPageNum(1);
            fetchPageData();
            setDelKey("");
        } catch (e: any) { setResJson({ error: e.message }); }
    };

    const goToScanPage = (num: number) => {
        if (num < 1 || num > scanTotalPages) return;
        setScanPageNum(num);
        setTableList(scanAllData.slice((num - 1) * pageSize, num * pageSize));
    };

    const goToPage = (num: number) => {
        if (num < 1 || num > totalPages) return;
        setPageNum(num);
    };

    const renderPagination = () => {
        if (tableList.length === 0 || isSingleMode) return null;
        const current = isScanMode ? scanPageNum : pageNum;
        const totalP = isScanMode ? scanTotalPages : totalPages;
        const totalItems = isScanMode ? scanAllData.length : total;
        const go = isScanMode ? goToScanPage : goToPage;

        const pages = [];
        let start = Math.max(1, current - 2);
        let end = Math.min(totalP, current + 2);
        for (let i = start; i <= end; i++) pages.push(i);

        return (
            <div className="flex items-center justify-between flex-wrap gap-4 mt-6 pt-4 border-t border-slate-700">
                <div className="flex items-center gap-2">
                    <button onClick={() => go(1)} disabled={current === 1} className="px-3 py-1.5 bg-slate-700 rounded disabled:opacity-50 hover:bg-slate-600 transition text-sm">首页</button>
                    <button onClick={() => go(current - 1)} disabled={current === 1} className="px-3 py-1.5 bg-slate-700 rounded disabled:opacity-50 hover:bg-slate-600 transition text-sm">上一页</button>
                    {pages.map((i) => (
                        <button key={i} onClick={() => go(i)} className={`px-3 py-1.5 rounded transition text-sm ${current === i ? "bg-violet-600 text-white" : "bg-slate-700 hover:bg-slate-600"}`}>{i}</button>
                    ))}
                    <button onClick={() => go(current + 1)} disabled={current >= totalP} className="px-3 py-1.5 bg-slate-700 rounded disabled:opacity-50 hover:bg-slate-600 transition text-sm">下一页</button>
                    <button onClick={() => go(totalP)} disabled={current >= totalP} className="px-3 py-1.5 bg-slate-700 rounded disabled:opacity-50 hover:bg-slate-600 transition text-sm">尾页</button>
                </div>
                <span className="text-sm text-slate-400">第 {current} 页 / 共 {totalP} 页 | 总计 {totalItems} 条</span>
            </div>
        );
    };

    return (
        <div className="min-h-screen bg-[#0f172a] text-slate-200 p-8">
            <div className="max-w-7xl mx-auto space-y-8">
                
                {/* Header 区域 */}
                <header className="flex justify-between items-center bg-slate-800/40 backdrop-blur-md p-6 rounded-2xl border border-slate-700/50 shadow-xl">
                    <div>
                        <h1 className="text-4xl font-extrabold bg-gradient-to-r from-sky-400 to-indigo-400 bg-clip-text text-transparent">
                            AeroKV Control Center
                        </h1>
                        <p className="text-slate-400 mt-1 text-sm">分布式存储与共识管理后台</p>
                    </div>
                    {/* 🌟 新增：权限管理按钮 */}
                    <button 
                        // onClick={() => router.push("/permission")}
                        onClick={() => router.push("/admin")}
                        className="px-6 py-2.5 bg-indigo-600/20 text-indigo-400 border border-indigo-500/30 rounded-xl hover:bg-indigo-600/30 transition-all font-semibold shadow-lg shadow-indigo-900/10"
                    >
                        🛡️ 权限管理
                    </button>
                </header>

                {/* 顶部控制区 */}
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                    {/* 初始化 & 控制 */}
                    <div className="bg-slate-800/50 backdrop-blur-sm p-6 rounded-2xl border border-slate-700/50 shadow-lg">
                        <h2 className="text-xl font-bold mb-6 flex items-center gap-2 text-sky-400">
                            <span className="w-2 h-6 bg-sky-500 rounded-full"></span> 🚀 系统控制
                        </h2>
                        <div className="space-y-4">
                            <button
                                onClick={handleInitAll}
                                disabled={loadingInit}
                                className="w-full py-3 bg-gradient-to-r from-sky-600 to-indigo-600 rounded-xl hover:from-sky-500 hover:to-indigo-500 disabled:opacity-60 transition-all font-bold shadow-lg shadow-sky-900/20"
                            >
                                {loadingInit ? "执行中..." : "系统一键初始化"}
                            </button>
                            <div className="flex gap-4">
                                <button onClick={() => handlePause(true)} className="flex-1 py-2.5 bg-rose-500/10 text-rose-400 border border-rose-500/20 rounded-lg hover:bg-rose-500/20 transition-all font-semibold">紧急暂停</button>
                                <button onClick={() => handlePause(false)} className="flex-1 py-2.5 bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 rounded-lg hover:bg-emerald-500/20 transition-all font-semibold">恢复运行</button>
                            </div>
                            {ctrlMsg && <p className="text-sm text-amber-300 font-medium px-2 italic">{ctrlMsg}</p>}
                            <pre className="p-4 bg-black/40 rounded-xl text-emerald-400 text-xs h-32 overflow-auto border border-slate-700 font-mono">
                                {initResult || "> 等待初始化操作..."}
                            </pre>
                        </div>
                    </div>

                    {/* 写入 & 删除 */}
                    <div className="bg-slate-800/50 backdrop-blur-sm p-6 rounded-2xl border border-slate-700/50 shadow-lg">
                        <h2 className="text-xl font-bold mb-6 flex items-center gap-2 text-emerald-400">
                            <span className="w-2 h-6 bg-emerald-500 rounded-full"></span> 📝 数据管理
                        </h2>
                        <div className="space-y-4">
                            <div className="grid grid-cols-2 gap-4">
                                <input value={writeKey} onChange={(e) => setWriteKey(e.target.value)} placeholder="写入 Key" className="bg-slate-900/50 border border-slate-700 rounded-xl px-4 py-3 outline-none focus:ring-2 focus:ring-emerald-500/50 transition-all" />
                                <input value={writeValue} onChange={(e) => setWriteValue(e.target.value)} placeholder="写入 Value" className="bg-slate-900/50 border border-slate-700 rounded-xl px-4 py-3 outline-none focus:ring-2 focus:ring-emerald-500/50 transition-all" />
                            </div>
                            <button onClick={handleUpsert} className="w-full py-3 bg-emerald-600 rounded-xl hover:bg-emerald-500 transition-all font-bold shadow-lg shadow-emerald-900/20">提交写入</button>
                            <div className="flex gap-3">
                                <input value={delKey} onChange={(e) => setDelKey(e.target.value)} placeholder="要删除的 Key" className="flex-1 bg-slate-900/50 border border-slate-700 rounded-xl px-4 py-3 outline-none focus:ring-2 focus:ring-rose-500/50 transition-all" />
                                <button onClick={handleDelete} className="px-8 py-3 bg-rose-600 rounded-xl hover:bg-rose-500 transition-all font-bold shadow-lg shadow-rose-900/20">删除</button>
                            </div>
                            <button onClick={async () => await mock()} className="w-full py-2.5 bg-slate-700/50 border border-slate-600 rounded-xl hover:bg-slate-700 transition-all text-slate-400 font-medium">自动添加数据测试 (Mock)</button>
                        </div>
                    </div>
                </div>

                {/* 查询 & 扫描 */}
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                    <div className="bg-slate-800/50 backdrop-blur-sm p-6 rounded-2xl border border-slate-700/50 shadow-lg">
                        <h2 className="text-lg font-bold mb-6 text-teal-300">🔍 单点检索</h2>
                        <div className="flex gap-3">
                            <input value={queryKey} onChange={(e) => setQueryKey(e.target.value)} placeholder="输入查询 Key" className="flex-1 bg-slate-900/50 border border-slate-700 rounded-xl px-4 py-3 outline-none focus:ring-2 focus:ring-teal-500/50" />
                            <button onClick={handleGetSingle} className="px-8 py-3 bg-teal-600 rounded-xl hover:bg-teal-500 transition-all font-bold">查询</button>
                        </div>
                        {merkleStatus && <p className={`mt-4 px-3 py-2 rounded-lg text-sm font-medium ${merkleStatus.includes("通过") ? "bg-emerald-500/10 text-emerald-400" : "bg-rose-500/10 text-rose-400"}`}>{merkleStatus}</p>}
                    </div>
                    <div className="bg-slate-800/50 backdrop-blur-sm p-6 rounded-2xl border border-slate-700/50 shadow-lg">
                        <h2 className="text-lg font-bold mb-6 text-violet-300">📃 范围扫描</h2>
                        <div className="grid grid-cols-2 gap-4">
                            <input value={scanStartKey} onChange={(e) => setScanStartKey(e.target.value)} placeholder="起始 Key" className="bg-slate-900/50 border border-slate-700 rounded-xl px-4 py-3 outline-none focus:ring-2 focus:ring-violet-500/50" />
                            <input type="number" value={scanLimit} onChange={(e) => setScanLimit(Number(e.target.value))} placeholder="限制条数" className="bg-slate-900/50 border border-slate-700 rounded-xl px-4 py-3 outline-none focus:ring-2 focus:ring-violet-500/50" />
                        </div>
                        <button onClick={handleScan} className="w-full mt-4 py-3 bg-violet-600 rounded-xl hover:bg-violet-500 transition-all font-bold">开始扫描任务</button>
                    </div>
                </div>

                {/* 主数据表格 */}
                <div className="bg-slate-800/50 backdrop-blur-md rounded-2xl border border-slate-700/50 shadow-2xl overflow-hidden">
                    <div className="p-6 border-b border-slate-700/50 flex justify-between items-center bg-slate-900/30">
                        <h2 className="text-2xl font-bold text-slate-100 flex items-center gap-3">
                            📊 {isScanMode ? "扫描结果" : "数据快照"}
                            {isScanMode && <span className="text-xs bg-violet-500/20 text-violet-400 px-2 py-1 rounded border border-violet-500/30">Scan Mode</span>}
                        </h2>
                        <div className="flex gap-3">
                            <button onClick={() => router.push("/visual")} className="px-5 py-2 bg-orange-600 rounded-xl hover:bg-orange-500 transition-all text-sm font-bold shadow-lg shadow-orange-900/20">Merkle 可视化</button>
                            <button onClick={() => { setIsScanMode(false); setIsSingleMode(false); setPageNum(1); fetchPageData(); }} className="px-5 py-2 bg-slate-700 rounded-xl hover:bg-slate-600 transition-all text-sm font-bold">刷新全表</button>
                        </div>
                    </div>
                    <div className="overflow-x-auto">
                        <table className="w-full text-left">
                            <thead className="bg-slate-900/50 text-slate-400 text-xs uppercase tracking-widest font-bold">
                                <tr>
                                    <th className="px-8 py-4">Key</th>
                                    <th className="px-8 py-4">Value</th>
                                    <th className="px-8 py-4 text-center">Merkle 证据</th>
                                </tr>
                            </thead>
                            <tbody className="divide-y divide-slate-700/30">
                                {tableList.length > 0 ? (
                                    tableList.map((item, idx) => (
                                        <tr key={idx} className="hover:bg-slate-700/20 transition-colors group">
                                            <td className="px-8 py-4 font-mono text-sky-400">{item.key}</td>
                                            <td className="px-8 py-4 font-mono text-slate-300">{item.value}</td>
                                            <td className="px-8 py-4 text-center">
                                                <button 
                                                    onClick={() => {
                                                        const hash = resJson.key_hashes?.[idx];
                                                        const lIdx = resJson.leaf_indices?.[idx];
                                                        saveProofData([item], [hash], [lIdx], resJson.proof, resJson.merkle_root);
                                                        router.push("/visual");
                                                    }}
                                                    className="text-xs font-bold text-indigo-400 hover:text-indigo-300 underline underline-offset-4 decoration-indigo-500/30"
                                                >
                                                    验证此条
                                                </button>
                                            </td>
                                        </tr>
                                    ))
                                ) : (
                                    <tr><td colSpan={3} className="px-8 py-16 text-center text-slate-500 italic">暂无集群数据记录</td></tr>
                                )}
                            </tbody>
                        </table>
                    </div>
                    <footer className="px-8 py-4 bg-slate-900/30">
                        {renderPagination()}
                    </footer>
                </div>

                {/* 接口监控日志 (收起状态) */}
                <details className="bg-slate-900/40 rounded-xl border border-slate-800 overflow-hidden">
                    <summary className="px-4 py-2 text-xs font-bold text-slate-500 cursor-pointer hover:bg-slate-800 transition-colors uppercase tracking-widest">
                        📡 查看实时接口响应监控
                    </summary>
                    <pre className="p-4 text-[10px] font-mono text-slate-400 max-h-60 overflow-auto border-t border-slate-800">
                        {resJson ? JSON.stringify(resJson, null, 2) : "// 无活跃数据流"}
                    </pre>
                </details>
            </div>
        </div>
    );
}