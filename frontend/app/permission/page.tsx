"use client";

import React, { useState } from 'react';
import { Shield, Save, CheckCircle2, Circle, UserPlus, Info } from 'lucide-react';

// 1. 定义与后端完全一致的 Action 类型
type Action =
    | "InitAuth" | "InitStorage" | "InitCounter" | "InitFee"
    | "RaftUpsert" | "RaftDelete" | "Get" | "Scan"
    | "Page" | "RaftPause" | "RaftFee";

// 2. 补全映射表：对应你 Rust 图片中的所有枚举项
const PERMISSION_MAP: { id: Action; label: string; desc: string }[] = [
    { id: "InitAuth", label: "超级管理员", desc: "初始化权限系统、授权或撤销其他用户权限" },
    { id: "InitStorage", label: "存储初始化", desc: "初始化底层存储引擎与相关配置" },
    { id: "InitCounter", label: "计数器初始化", desc: "设置或重置全局计数器状态" },
    { id: "InitFee", label: "手续费初始化", desc: "定义系统交易的基础手续费参数" },
    { id: "RaftUpsert", label: "数据写入", desc: "允许新增或修改数据库中的 Key-Value 记录" },
    { id: "RaftDelete", label: "数据删除", desc: "允许物理删除数据库中的特定数据" },
    { id: "Get", label: "读取权限", desc: "允许通过精确 Key 获取单条记录内容" },
    { id: "Scan", label: "范围扫描", desc: "允许执行范围查询 (Range Scan) 操作" },
    { id: "Page", label: "分页查询", desc: "允许执行带分页参数的数据检索" },
    { id: "RaftPause", label: "系统暂停", desc: "紧急情况下暂停状态机处理，用于维护" },
    { id: "RaftFee", label: "费率调整", desc: "通过共识动态调整系统手续费标准" },
];

export default function PermissionPage() {
    // 状态管理
    const [targetPubkey, setTargetPubkey] = useState(""); // 目标用户公钥
    const [selectedActions, setSelectedActions] = useState<Action[]>(["Get", "Scan"]);
    const [isSubmitting, setIsSubmitting] = useState(false);

    // 切换权限逻辑
    const handleToggle = (id: Action) => {
        setSelectedActions(prev =>
            prev.includes(id) ? prev.filter(a => a !== id) : [...prev, id]
        );
    };

    // 提交到后端
    const savePermissions = async () => {
        if (!targetPubkey || targetPubkey.length < 32) {
            alert("请输入有效的目标用户公钥");
            return;
        }

        setIsSubmitting(true);
        
        // 模拟调用 backend 逻辑
        console.log("发起的 Raft 提案内容:", {
            target: targetPubkey,
            permissions: selectedActions
        });

        setTimeout(() => {
            setIsSubmitting(false);
            alert(`成功为用户 ${targetPubkey.slice(0, 8)}... 提交权限变更提案！`);
        }, 1000);
    };

    return (
        <div className="min-h-screen bg-slate-950 text-slate-200 p-6 md:p-10 font-sans">
            <div className="max-w-5xl mx-auto">
                {/* 头部展示 */}
                <div className="flex flex-col md:flex-row items-start md:items-center justify-between mb-8 gap-6">
                    <div>
                        <h1 className="text-3xl font-bold flex items-center gap-3 text-white">
                            <Shield className="text-cyan-500" /> AeroKV 权限分发中心
                        </h1>
                        <p className="text-slate-500 mt-2">
                            变更将作为 <span className="text-cyan-900 font-mono">KvOp::SyncGrant</span> 提交给 Raft 集群
                        </p>
                    </div>
                    <button
                        onClick={savePermissions}
                        disabled={isSubmitting}
                        className="w-full md:w-auto bg-cyan-600 hover:bg-cyan-500 text-white px-8 py-3 rounded-xl font-bold shadow-lg shadow-cyan-900/20 transition-all flex items-center justify-center gap-2 disabled:opacity-50"
                    >
                        <Save size={18} /> {isSubmitting ? "共识确认中..." : "提交授权提案"}
                    </button>
                </div>

                {/* 用户输入区 */}
                <div className="bg-slate-900/50 border border-slate-800 rounded-2xl p-6 mb-8">
                    <div className="flex items-center gap-2 mb-4 text-cyan-400">
                        <UserPlus size={20} />
                        <span className="font-semibold">目标用户身份</span>
                    </div>
                    <div className="relative">
                        <input
                            type="text"
                            value={targetPubkey}
                            onChange={(e) => setTargetPubkey(e.target.value)}
                            placeholder="输入 Solana 公钥 (Base58 格式)"
                            className="w-full bg-slate-950 border border-slate-700 rounded-xl px-4 py-4 text-cyan-50 font-mono focus:outline-none focus:border-cyan-500 transition-colors"
                        />
                        <div className="absolute right-4 top-4 text-slate-600">
                            <Info size={20} />
                        </div>
                    </div>
                    <p className="text-slate-500 text-xs mt-3">
                        提示：请务必核对公钥地址，权限变更一旦通过 Raft 确认将立即生效。
                    </p>
                </div>

                {/* 权限选择区 */}
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {PERMISSION_MAP.map((item) => {
                        const isChecked = selectedActions.includes(item.id);
                        return (
                            <div
                                key={item.id}
                                onClick={() => handleToggle(item.id)}
                                className={`group p-5 rounded-2xl border-2 transition-all duration-200 cursor-pointer ${
                                    isChecked
                                        ? "border-cyan-500 bg-cyan-500/10 shadow-inner shadow-cyan-500/10"
                                        : "border-slate-800 bg-slate-900/30 hover:border-slate-700"
                                }`}
                            >
                                <div className="flex justify-between items-start mb-3">
                                    <h3 className={`font-bold transition-colors ${isChecked ? "text-cyan-400" : "text-slate-300"}`}>
                                        {item.label}
                                    </h3>
                                    {isChecked ? (
                                        <CheckCircle2 className="text-cyan-500 animate-in zoom-in-75 duration-200" size={22} />
                                    ) : (
                                        <Circle className="text-slate-700 group-hover:text-slate-500" size={22} />
                                    )}
                                </div>
                                <p className="text-xs text-slate-500 leading-relaxed min-h-[32px]">
                                    {item.desc}
                                </p>
                                <div className="mt-4 pt-3 border-t border-slate-800/50">
                                    <span className="text-[10px] font-mono text-slate-600 uppercase tracking-wider">
                                        Action::{item.id}
                                    </span>
                                </div>
                            </div>
                        );
                    })}
                </div>
            </div>
        </div>
    );
}