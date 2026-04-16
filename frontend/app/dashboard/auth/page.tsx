"use client";

import { useState } from "react";

export default function AuthPage() {
    const [newAdmin, setNewAdmin] = useState("");
    const [pause, setPause] = useState(false);
    const [msg, setMsg] = useState("");

    const setAdmin = async () => {
        try {
            const res = await fetch("/api/auth/set-admin", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ new_admin: newAdmin }),
            });
            setMsg(await res.text());
        } catch (err) {
            setMsg("请求失败");
        }
    };

    const togglePause = async () => {
        try {
            const res = await fetch("/api/auth/set-pause", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ pause }),
            });
            setMsg(await res.text());
        } catch (err) {
            setMsg("请求失败");
        }
    };

    return (
        <div className="min-h-screen bg-[#0f172a] text-white p-8">
            <h1 className="text-2xl font-bold mb-6 text-blue-400">权限管理</h1>

            <div className="max-w-xl bg-[#1e293b] p-6 rounded-2xl space-y-6">
                <div>
                    <label className="block mb-2 text-sm">新管理员公钥</label>
                    <input
                        value={newAdmin}
                        onChange={(e) => setNewAdmin(e.target.value)}
                        className="w-full p-2 rounded bg-gray-800 border border-gray-700"
                    />
                    <button
                        onClick={setAdmin}
                        className="mt-3 bg-purple-500 px-4 py-2 rounded"
                    >
                        设置管理员
                    </button>
                </div>

                <div className="flex items-center gap-3">
                    <input
                        type="checkbox"
                        checked={pause}
                        onChange={(e) => setPause(e.target.checked)}
                        id="pause"
                    />
                    <label htmlFor="pause">暂停合约</label>
                    <button
                        onClick={togglePause}
                        className="ml-3 bg-red-500 px-4 py-2 rounded"
                    >
                        确认
                    </button>
                </div>

                {msg && <div className="p-3 bg-gray-800 rounded text-sm">{msg}</div>}
            </div>
        </div>
    );
}