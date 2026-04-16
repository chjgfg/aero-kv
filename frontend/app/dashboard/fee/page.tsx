"use client";

import { useState } from "react";

export default function FeePage() {
    const [fee, setFee] = useState("");
    const [msg, setMsg] = useState("");

    const setNewFee = async () => {
        try {
            const res = await fetch("/api/fee/set-fee", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ fee: Number(fee) }),
            });
            setMsg(await res.text());
        } catch (err) {
            setMsg("请求失败");
        }
    };

    return (
        <div className="min-h-screen bg-[#0f172a] text-white p-8">
            <h1 className="text-2xl font-bold mb-6 text-blue-400">手续费管理</h1>

            <div className="max-w-xl bg-[#1e293b] p-6 rounded-2xl space-y-4">
                <div>
                    <label className="block mb-2 text-sm">新手续费（lamports）</label>
                    <input
                        type="number"
                        value={fee}
                        onChange={(e) => setFee(e.target.value)}
                        className="w-full p-2 rounded bg-gray-800 border border-gray-700"
                    />
                </div>

                <button
                    onClick={setNewFee}
                    className="bg-yellow-500 px-4 py-2 rounded text-black font-semibold"
                >
                    设置手续费
                </button>

                {msg && <div className="p-3 bg-gray-800 rounded text-sm">{msg}</div>}
            </div>
        </div>
    );
}