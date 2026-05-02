// "use client";

// import { useState, useEffect } from "react";
// import { useRouter } from "next/navigation";
// import { grant, revoke, admin_get, admin_page } from "@/utils/http"; // 请确保 http.ts 中有这些导出

// // 对应你图片中的 Action 类型
// const ALL_ACTIONS = [
//     "InitAuth", "InitStorage", "InitCounter", "InitFee",
//     "RaftUpsert", "RaftDelete", "Get", "Scan",
//     "Page", "RaftPause", "RaftFee"
// ];

// type PermissionItem = {
//     pubkey: string;
//     permissions: string[];
// };

// export default function AuthManagePage() {
//     const router = useRouter();

//     // --- 状态管理 ---
//     const [tableList, setTableList] = useState<PermissionItem[]>([]);
//     const [pageNum, setPageNum] = useState(1);
//     const [pageSize] = useState(10);
//     const [totalPages, setTotalPages] = useState(1);
//     const [total, setTotal] = useState(0);

//     // 授权表单状态
//     const [adminKey, setAdminKey] = useState("");
//     const [targetUserKey, setTargetUserKey] = useState("");
//     const [selectedActions, setSelectedActions] = useState<string[]>([]);

//     // 删除/查询状态
//     const [revokeKey, setRevokeKey] = useState("");
//     const [searchKey, setSearchKey] = useState("");

//     // 结果反馈
//     const [msg, setMsg] = useState("");

//     useEffect(() => {
//         fetchAuthData();
//     }, [pageNum]);

//     const fetchAuthData = async () => {
//         try {
//             // 这里假设你的 page 接口也能返回权限列表，或者你有专门的 auth_page 接口
//             // const res = await page(pageNum, pageSize);
//             // setTotal(res.total || 0);
//             // setTotalPages(Math.ceil((res.total || 0) / pageSize));
//             // if (Array.isArray(res?.auth_list)) {
//             //     setTableList(res.auth_list);
//             // }
//         } catch (e: any) {
//             setMsg(`❌ 加载失败: ${e.message}`);
//         }
//     };

//     const handleGrant = async () => {
//         if (!targetUserKey || selectedActions.length === 0) {
//             setMsg("⚠️ 请填写完整授权信息");
//             return;
//         }
//         try {
//             // await grant(adminKey, targetUserKey, selectedActions);
//             // setMsg("✅ 权限授予成功");
//             // fetchAuthData();
//         } catch (e: any) {
//             setMsg(`❌ 授权失败: ${e.message}`);
//         }
//     };

//     const handleRevoke = async () => {
//         try {
//             // await revoke(revokeKey);
//             // setMsg("✅ 权限已回收");
//             // fetchAuthData();
//         } catch (e: any) {
//             setMsg(`❌ 回收失败: ${e.message}`);
//         }
//     };

//     const toggleAction = (action: string) => {
//         setSelectedActions(prev =>
//             prev.includes(action) ? prev.filter(a => a !== action) : [...prev, action]
//         );
//     };

//     return (
//         <div className="min-h-screen bg-[#0b0f1a] text-slate-100 p-8">
//             <div className="max-w-7xl mx-auto space-y-8">

//                 {/* Header */}
//                 <div className="flex justify-between items-center bg-slate-800/40 p-6 rounded-2xl border border-slate-700/50 backdrop-blur-md">
//                     <div>
//                         <h1 className="text-3xl font-black bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">
//                             🛡️ 权限安全中心
//                         </h1>
//                         <p className="text-slate-400 text-sm mt-1">管理 AeroKV 集群的访问控制列表 (ACL)</p>
//                     </div>
//                     <button
//                         onClick={() => router.push("/")}
//                         className="px-6 py-2 bg-slate-700/50 rounded-xl hover:bg-slate-600 border border-slate-600 transition-all text-sm font-bold"
//                     >
//                         返回存储管理
//                     </button>
//                 </div>

//                 <div className="grid grid-cols-1 lg:grid-cols-12 gap-8">

//                     {/* 左侧：操作面板 */}
//                     <div className="lg:col-span-4 space-y-6">

//                         {/* 授予权限 */}
//                         <div className="bg-slate-800/40 p-6 rounded-2xl border border-slate-700/50">
//                             <h2 className="text-lg font-bold mb-6 text-indigo-400 flex items-center gap-2">
//                                 <span className="w-1.5 h-5 bg-indigo-500 rounded-full"></span> 授予权限
//                             </h2>
//                             <div className="space-y-4">
//                                 <input
//                                     value={adminKey} onChange={e => setAdminKey(e.target.value)}
//                                     placeholder="管理员公钥"
//                                     className="w-full bg-slate-900/50 border border-slate-700 rounded-xl px-4 py-3 outline-none focus:border-indigo-500 transition-all text-sm"
//                                 />
//                                 <input
//                                     value={targetUserKey} onChange={e => setTargetUserKey(e.target.value)}
//                                     placeholder="目标用户公钥"
//                                     className="w-full bg-slate-900/50 border border-slate-700 rounded-xl px-4 py-3 outline-none focus:border-indigo-500 transition-all text-sm"
//                                 />

//                                 <div className="space-y-2">
//                                     <label className="text-[10px] text-slate-500 uppercase font-bold tracking-widest ml-1">选择权限 (Actions)</label>
//                                     <div className="flex flex-wrap gap-2 p-3 bg-black/20 rounded-xl border border-slate-800">
//                                         {ALL_ACTIONS.map(action => (
//                                             <button
//                                                 key={action}
//                                                 onClick={() => toggleAction(action)}
//                                                 className={`px-3 py-1 rounded-md text-[10px] font-bold transition-all border ${selectedActions.includes(action)
//                                                     ? "bg-indigo-600 border-indigo-400 text-white"
//                                                     : "bg-slate-800 border-slate-700 text-slate-500 hover:border-slate-500"
//                                                     }`}
//                                             >
//                                                 {action}
//                                             </button>
//                                         ))}
//                                     </div>
//                                 </div>

//                                 <button
//                                     onClick={handleGrant}
//                                     className="w-full py-3 bg-indigo-600 rounded-xl font-bold hover:bg-indigo-500 transition-all shadow-lg shadow-indigo-900/20"
//                                 >
//                                     确认授予
//                                 </button>
//                             </div>
//                         </div>

//                         {/* 权限回收 */}
//                         <div className="bg-slate-800/40 p-6 rounded-2xl border border-slate-700/50">
//                             <h2 className="text-lg font-bold mb-6 text-rose-400 flex items-center gap-2">
//                                 <span className="w-1.5 h-5 bg-rose-500 rounded-full"></span> 权限回收
//                             </h2>
//                             <div className="flex gap-2">
//                                 <input
//                                     value={revokeKey} onChange={e => setRevokeKey(e.target.value)}
//                                     placeholder="用户公钥"
//                                     className="flex-1 bg-slate-900/50 border border-slate-700 rounded-xl px-4 py-2 outline-none focus:border-rose-500 text-sm"
//                                 />
//                                 <button onClick={handleRevoke} className="px-6 py-2 bg-rose-600/20 text-rose-400 border border-rose-600/30 rounded-xl hover:bg-rose-600/40 transition-all font-bold text-xs">回收</button>
//                             </div>
//                         </div>

//                         {msg && (
//                             <div className="p-4 bg-slate-900/80 border border-slate-700 rounded-xl text-sm font-medium animate-pulse">
//                                 {msg}
//                             </div>
//                         )}
//                     </div>

//                     {/* 右侧：表格展示 */}
//                     <div className="lg:col-span-8 space-y-6">
//                         <div className="bg-slate-800/40 rounded-2xl border border-slate-700/50 overflow-hidden shadow-2xl">
//                             <div className="p-6 border-b border-slate-700/50 bg-slate-900/30 flex justify-between items-center">
//                                 <h2 className="font-bold text-slate-100 flex items-center gap-2">
//                                     📋 权限名单 <span className="text-xs text-slate-500 font-normal ml-2">Total: {total}</span>
//                                 </h2>
//                                 <div className="flex gap-2">
//                                     <input
//                                         placeholder="搜索公钥..."
//                                         className="bg-slate-900/50 border border-slate-700 rounded-lg px-3 py-1 text-xs outline-none focus:border-indigo-500"
//                                     />
//                                     <button onClick={fetchAuthData} className="p-2 bg-slate-700/50 rounded-lg border border-slate-600 hover:bg-slate-600 transition-all text-xs">🔄</button>
//                                 </div>
//                             </div>
//                             <div className="overflow-x-auto">
//                                 <table className="w-full text-sm">
//                                     <thead>
//                                         <tr className="text-slate-400 border-b border-slate-700/50 bg-slate-900/10">
//                                             <th className="px-6 py-4 text-left font-bold uppercase tracking-widest text-[10px]">User Public Key</th>
//                                             <th className="px-6 py-4 text-left font-bold uppercase tracking-widest text-[10px]">Actions / Permissions</th>
//                                         </tr>
//                                     </thead>
//                                     <tbody className="divide-y divide-slate-700/30">
//                                         {tableList.length > 0 ? tableList.map((item, idx) => (
//                                             <tr key={idx} className="hover:bg-indigo-400/5 transition-colors group">
//                                                 <td className="px-6 py-4 font-mono text-indigo-400 text-xs truncate max-w-[200px]">{item.pubkey}</td>
//                                                 <td className="px-6 py-4">
//                                                     <div className="flex flex-wrap gap-1">
//                                                         {item.permissions.map(p => (
//                                                             <span key={p} className="px-2 py-0.5 bg-slate-700/50 text-slate-300 text-[9px] rounded border border-slate-600">
//                                                                 {p}
//                                                             </span>
//                                                         ))}
//                                                     </div>
//                                                 </td>
//                                             </tr>
//                                         )) : (
//                                             <tr><td colSpan={2} className="px-6 py-12 text-center text-slate-500 font-mono italic">-- No permission records found --</td></tr>
//                                         )}
//                                     </tbody>
//                                 </table>
//                             </div>

//                             {/* 分页按钮 */}
//                             <div className="p-4 bg-slate-900/30 border-t border-slate-700/50 flex items-center justify-between">
//                                 <div className="flex gap-2">
//                                     <button onClick={() => setPageNum(p => Math.max(1, p - 1))} disabled={pageNum === 1} className="px-4 py-1.5 bg-slate-800 rounded-lg border border-slate-700 text-xs hover:bg-slate-700 disabled:opacity-30">Prev</button>
//                                     <button onClick={() => setPageNum(p => Math.min(totalPages, p + 1))} disabled={pageNum >= totalPages} className="px-4 py-1.5 bg-slate-800 rounded-lg border border-slate-700 text-xs hover:bg-slate-700 disabled:opacity-30">Next</button>
//                                 </div>
//                                 <span className="text-[10px] font-mono text-slate-500 uppercase">
//                                     Page {pageNum} / {totalPages}
//                                 </span>
//                             </div>
//                         </div>
//                     </div>
//                 </div>
//             </div>
//         </div>
//     );
// }

"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import { grant, revoke, admin_get, admin_page, Action } from "@/utils/http";

const ALL_ACTIONS = [
    "InitAuth", "InitStorage", "InitCounter", "InitFee",
    "RaftUpsert", "RaftDelete", "Get", "Scan",
    "Page", "RaftPause", "RaftFee"
];

type PermissionItem = {
    pubkey: string;
    permissions: string[];
};

export default function AuthManagePage() {
    const router = useRouter();

    // --- 状态管理 ---
    const [tableList, setTableList] = useState<PermissionItem[]>([]);
    const [pageNum, setPageNum] = useState(1);
    const [pageSize] = useState(10);
    const [total, setTotal] = useState(0);
    const [totalPages, setTotalPages] = useState(1);

    // 假分页相关
    const [isSearchMode, setIsSearchMode] = useState(false);
    const [allSearchData, setAllSearchData] = useState<PermissionItem[]>([]);

    // 表单与搜索
    const [adminKey, setAdminKey] = useState("");
    const [targetUserKey, setTargetUserKey] = useState("");
    const [selectedActions, setSelectedActions] = useState<Action[]>([]);
    const [searchKey, setSearchKey] = useState("");
    const [msg, setMsg] = useState("");

    // 初始化加载
    useEffect(() => {
        if (!isSearchMode) fetchAuthData();
    }, [pageNum, isSearchMode]);

    // 1. 真分页加载 (admin_page)
    const fetchAuthData = async () => {
        try {
            const res = await admin_page(pageNum, pageSize);
            setTotal(res.total || 0);
            setTotalPages(Math.ceil((res.total || 0) / pageSize));
            setTableList(res.auth_list || []);
        } catch (e: any) {
            setMsg(`❌ 加载失败: ${e.message}`);
        }
    };

    // 2. 搜索并开启假分页 (admin_get)
    const handleSearch = async () => {
        if (!searchKey) {
            setIsSearchMode(false);
            setPageNum(1);
            return;
        }
        try {
            const res = await admin_get(searchKey, 10); // 假设搜索上限100条
            const list = res.auth_list || [];
            setAllSearchData(list);
            setIsSearchMode(true);
            setPageNum(1);
            setTotal(list.length);
            setTotalPages(Math.ceil(list.length / pageSize));
            // 初始截取第一页
            setTableList(list.slice(0, pageSize));
        } catch (e: any) {
            setMsg(`❌ 搜索失败: ${e.message}`);
        }
    };

    // 假分页跳转逻辑
    const goToFakePage = (num: number) => {
        setPageNum(num);
        const start = (num - 1) * pageSize;
        setTableList(allSearchData.slice(start, start + pageSize));
    };

    const handleGrant = async () => {
        try {
            await grant(targetUserKey, selectedActions as Action[]);
            setMsg("✅ 权限授予成功");
            fetchAuthData();
        } catch (e: any) { setMsg(`❌ 授权失败: ${e.message}`); }
    };

    const handleRevoke = async (key: string) => {
        try {
            await revoke(key);
            setMsg("✅ 权限已回收");
            if (isSearchMode) handleSearch(); else fetchAuthData();
        } catch (e: any) { setMsg(`❌ 回收失败: ${e.message}`); }
    };

    const toggleAction = (action: Action) => {
        setSelectedActions(prev => prev.includes(action) ? prev.filter(a => a !== action) : [...prev, action]);
    };

    // --- 分页渲染组件 ---
    const renderPagination = () => {
        if (totalPages <= 1) return null;
        const pages = [];
        for (let i = Math.max(1, pageNum - 2); i <= Math.min(totalPages, pageNum + 2); i++) {
            pages.push(i);
        }
        const jump = isSearchMode ? goToFakePage : setPageNum;

        return (
            <div className="flex items-center gap-2">
                <button onClick={() => jump(1)} disabled={pageNum === 1} className="px-3 py-1 bg-slate-700 rounded disabled:opacity-30 text-xs">首页</button>
                {pages.map(i => (
                    <button key={i} onClick={() => jump(i)} className={`px-3 py-1 rounded text-xs ${pageNum === i ? "bg-indigo-600" : "bg-slate-700"}`}>{i}</button>
                ))}
                <button onClick={() => jump(totalPages)} disabled={pageNum === totalPages} className="px-3 py-1 bg-slate-700 rounded disabled:opacity-30 text-xs">末页</button>
            </div>
        );
    };

    return (
        <div className="min-h-screen bg-[#0b0f1a] text-slate-100 p-8">
            <div className="max-w-7xl mx-auto space-y-8">
                {/* Header */}
                <div className="flex justify-between items-center bg-slate-800/40 p-6 rounded-2xl border border-slate-700/50 backdrop-blur-md">
                    <h1 className="text-3xl font-black bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">🛡️ 权限安全中心</h1>
                    <button onClick={() => router.push("/")} className="px-6 py-2 bg-slate-700/50 rounded-xl hover:bg-slate-600 border border-slate-600 transition-all text-sm font-bold">返回存储管理</button>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-12 gap-8">
                    {/* 左侧面板 */}
                    <div className="lg:col-span-4 space-y-6">
                        <div className="bg-slate-800/40 p-6 rounded-2xl border border-slate-700/50 shadow-xl">
                            <h2 className="text-lg font-bold mb-6 text-indigo-400 flex items-center gap-2">授予权限</h2>
                            <div className="space-y-4">
                                <input value={adminKey} onChange={e => setAdminKey(e.target.value)} placeholder="管理员公钥" className="w-full bg-slate-900/50 border border-slate-700 rounded-xl px-4 py-3 outline-none focus:border-indigo-500 text-sm" />
                                <input value={targetUserKey} onChange={e => setTargetUserKey(e.target.value)} placeholder="目标用户公钥" className="w-full bg-slate-900/50 border border-slate-700 rounded-xl px-4 py-3 outline-none focus:border-indigo-500 text-sm" />
                                <div className="flex flex-wrap gap-2 p-3 bg-black/20 rounded-xl border border-slate-800">
                                    {ALL_ACTIONS.map((a) => {
                                        // 🌟 关键点：将 a 断言为 Action
                                        const action = a as Action;

                                        return (
                                            <button
                                                key={action}
                                                onClick={() => toggleAction(action)}
                                                className={`px-2 py-1 rounded text-[10px] border ${selectedActions.includes(action)
                                                        ? "bg-indigo-600 border-indigo-400"
                                                        : "bg-slate-800 border-slate-700"
                                                    }`}
                                            >
                                                {action}
                                            </button>
                                        );
                                    })}
                                </div>
                                <button onClick={handleGrant} className="w-full py-3 bg-indigo-600 rounded-xl font-bold hover:bg-indigo-500 shadow-lg shadow-indigo-900/20">确认授予</button>
                            </div>
                        </div>
                        {msg && <div className="p-4 bg-slate-900/80 border border-slate-700 rounded-xl text-sm animate-pulse">{msg}</div>}
                    </div>

                    {/* 右侧表格 */}
                    <div className="lg:col-span-8 space-y-6">
                        <div className="bg-slate-800/40 rounded-2xl border border-slate-700/50 overflow-hidden shadow-2xl">
                            <div className="p-6 border-b border-slate-700/50 bg-slate-900/30 flex justify-between items-center">
                                {/* <h2 className="font-bold flex items-center gap-2">📋 权限名单 <span className="text-[10px] bg-indigo-500/20 text-indigo-400 px-2 rounded">Hits: {total}</span></h2> */}
                                <div className="flex items-center gap-4">
                                    <h2 className="font-bold flex items-center gap-2 text-slate-100">
                                        📋 权限名单
                                        <span className="text-[10px] bg-indigo-500/20 text-indigo-400 px-2 rounded">Hits: {total}</span>
                                    </h2>

                                    {/* 🌟 新增：刷新全表按钮 */}
                                    <button
                                        onClick={() => {
                                            setIsSearchMode(false);
                                            setPageNum(1);
                                            fetchAuthData(); // 内部会调用 admin_page(1, 10)
                                        }}
                                        className="px-3 py-1 bg-sky-500/10 text-sky-400 border border-sky-500/20 rounded-lg hover:bg-sky-500/20 transition-all text-[10px] font-bold uppercase tracking-widest"
                                    >
                                        刷新全表
                                    </button>
                                </div>
                                <div className="flex gap-2">
                                    <input value={searchKey} onChange={e => setSearchKey(e.target.value)} placeholder="搜索公钥..." className="bg-slate-900/50 border border-slate-700 rounded-lg px-3 py-1 text-xs outline-none focus:border-indigo-500" />
                                    <button onClick={handleSearch} className="px-3 py-1 bg-indigo-600 rounded-lg text-xs font-bold">搜索</button>
                                </div>
                            </div>
                            <div className="overflow-x-auto text-xs">
                                <table className="w-full">
                                    <thead>
                                        <tr className="text-slate-400 border-b border-slate-700/50 bg-slate-900/10">
                                            <th className="px-6 py-4 text-left font-bold uppercase tracking-widest">公钥</th>
                                            <th className="px-6 py-4 text-left font-bold uppercase tracking-widest">权限</th>
                                            <th className="px-6 py-4 text-center font-bold uppercase tracking-widest">操作</th>
                                        </tr>
                                    </thead>
                                    <tbody className="divide-y divide-slate-700/30">
                                        {tableList.map((item, idx) => (
                                            <tr key={idx} className="hover:bg-indigo-400/5 transition-all group">
                                                <td className="px-6 py-4 font-mono text-indigo-400 truncate max-w-[150px]">{item.pubkey}</td>
                                                <td className="px-6 py-4">
                                                    <div className="flex flex-wrap gap-1">
                                                        {item.permissions.map(p => <span key={p} className="px-1.5 py-0.5 bg-slate-700/50 text-slate-300 rounded border border-slate-600">{p}</span>)}
                                                    </div>
                                                </td>
{/* 🌟 核心修改：去掉 opacity-0 和 group-hover 相关类名，改为始终显示 */}
            <td className="px-6 py-4">
                <div className="flex gap-2 justify-center">
                    <button 
                        onClick={() => setTargetUserKey(item.pubkey)} 
                        className="px-3 py-1.5 bg-emerald-600/20 text-emerald-400 rounded-lg border border-emerald-600/30 hover:bg-emerald-600/40 transition-all font-bold text-[10px] whitespace-nowrap"
                    >
                        授权
                    </button>
                    <button 
                        onClick={() => handleRevoke(item.pubkey)} 
                        className="px-3 py-1.5 bg-rose-600/20 text-rose-400 rounded-lg border border-rose-600/30 hover:bg-rose-600/40 transition-all font-bold text-[10px] whitespace-nowrap"
                    >
                        收回
                    </button>
                </div>
            </td>
                                            </tr>
                                        ))}
                                    </tbody>
                                </table>
                            </div>
                            <div className="p-4 bg-slate-900/30 border-t border-slate-700/50 flex items-center justify-between">
                                {renderPagination()}
                                <span className="text-[10px] font-mono text-slate-500 uppercase">Page {pageNum} / {totalPages}</span>
                            </div>
                        </div>
                    </div>


                </div>
            </div>
        </div>
    );
}