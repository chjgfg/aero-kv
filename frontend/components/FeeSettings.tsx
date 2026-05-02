"use client";

import { setFee } from "@/utils/http";
import { useState } from "react";

interface FeeSettingsProps {
    isOpen: boolean;
    onClose: () => void;
}

export default function FeeSettings({ isOpen, onClose }: FeeSettingsProps) {
    // 🌟 统一管理费用表单状态
    const [fees, setFees] = useState({
        base_fee: 1000000000,
        fee_per_byte: 1000000000,
        scan_fee_per_item: 1000000000,
    });
    const [loading, setLoading] = useState(false);

    const handleUpdate = async () => {
        setLoading(true);
        // 🌟 这里对接你的后端接口，提交 fees 对象
        console.log("提交 Raft 提案:", fees);
        await setFee(fees.base_fee, fees.fee_per_byte, fees.scan_fee_per_item);
        // 模拟请求完成
        setTimeout(() => {
            setLoading(false);
            onClose();
            setFees({ base_fee: 0, fee_per_byte: 0, scan_fee_per_item: 0 });
        }, 1000);
    };

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-[9999] flex items-center justify-center p-4">
            {/* 背景遮罩 */}
            <div
                className="absolute inset-0 bg-slate-950/80 backdrop-blur-sm animate-in fade-in duration-200"
                onClick={onClose}
            />

            {/* 弹窗主体 */}
            <div className="relative w-full max-w-md bg-slate-900 border border-slate-700 rounded-3xl p-8 shadow-2xl animate-in zoom-in-95 duration-200">
                <div className="flex justify-between items-center mb-6">
                    <h2 className="text-xl font-bold text-white flex items-center gap-2">
                        💰 计费参数设置
                    </h2>
                    <button onClick={onClose} className="text-slate-500 hover:text-white transition-colors">
                        <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                <div className="space-y-5">
                    {/* 🌟 传入 value 和 onChange */}
                    <FeeInput 
                        label="基础费用 (Base Fee)" 
                        unit="lamports" 
                        value={fees.base_fee}
                        onChange={(val) => setFees({ ...fees, base_fee: val })}
                    />
                    <FeeInput 
                        label="字节单价 (Per Byte)" 
                        unit="lamports/B" 
                        value={fees.fee_per_byte}
                        onChange={(val) => setFees({ ...fees, fee_per_byte: val })}
                    />
                    <FeeInput 
                        label="扫描单价 (Per Item)" 
                        unit="lamports/item" 
                        value={fees.scan_fee_per_item}
                        onChange={(val) => setFees({ ...fees, scan_fee_per_item: val })}
                    />

                    <div className="pt-4 flex gap-3">
                        <button
                            onClick={() => {
                                onClose(); 
                                setFees({ base_fee: 0, fee_per_byte: 0, scan_fee_per_item: 0 });
                                setLoading(false);
                            }}
                            className="flex-1 py-3 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-xl font-bold transition-all"
                        >
                            取消
                        </button>
                        <button
                            onClick={handleUpdate}
                            disabled={loading}
                            className="flex-1 py-3 bg-amber-600 hover:bg-amber-500 disabled:opacity-50 text-white rounded-xl font-bold transition-all shadow-lg shadow-amber-900/40"
                        >
                            {loading ? "提交中..." : "更新集群参数"}
                        </button>
                    </div>
                    <p className="text-[10px] text-slate-500 text-center">
                        * 提交后将发起 Raft 提案，需集群过半数节点达成共识后生效。
                    </p>
                </div>
            </div>
        </div>
    );
}

interface FeeInputProps {
    label: string;
    unit: string;
    value: number;
    onChange: (val: number) => void;
}

function FeeInput({ label, unit, value, onChange }: FeeInputProps) {
    return (
        <div>
            <label className="text-xs text-slate-400 block mb-2 px-1">{label}</label>
            <div className="relative">
                <input
                    type="text" // 🌟 改为 text，配合正则过滤，体验比 number 更好
                    value={value === 0 ? "" : value} // 🌟 如果是 0 就显示空，方便直接输入
                    onChange={(e) => {
                        const val = e.target.value;
                        // 🌟 只允许输入纯数字
                        if (val === "" || /^\d+$/.test(val)) {
                            onChange(val === "" ? 0 : Number(val));
                        }
                    }}
                    placeholder="0"
                    className="w-full bg-slate-950/50 border border-slate-700 rounded-xl px-4 py-3 text-amber-400 focus:border-amber-500 focus:ring-1 focus:ring-amber-500 outline-none transition-all"
                />
                <span className="absolute right-4 top-3.5 text-[10px] text-slate-600 font-mono pointer-events-none">
                    {unit}
                </span>
            </div>
        </div>
    );
}