"use client";

import { useState } from "react";

export default function KVPage() {
    const [key, setKey] = useState("");
    const [value, setValue] = useState("");
    const [result, setResult] = useState("");

    //  Upsert
    const handleUpsert = async () => {
        try {
            const res = await fetch("http://192.168.40.131/kv/upsert", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ key, value }),
            });
            const data = await res.text();
            setResult(data);
        } catch (err) {
            setResult("请求失败");
        }
    };

    // Get
    const handleGet = async () => {
        try {
            const res = await fetch(`http://192.168.40.131/kv/get?key=${key}`);
            const data = await res.text();
            setResult(data);
        } catch (err) {
            setResult("请求失败");
        }
    };

    return (
        <div className="min-h-screen bg-[#0f172a] text-white p-8">
            <h1 className="text-2xl font-bold mb-6 text-blue-400">KV 数据管理</h1>

            <div className="max-w-xl bg-[#1e293b] p-6 rounded-2xl">
                <div className="mb-4">
                    <label className="block mb-2 text-sm">Key</label>
                    <input
                        value={key}
                        onChange={(e) => setKey(e.target.value)}
                        className="w-full p-2 rounded bg-gray-800 border border-gray-700"
                    />
                </div>

                <div className="mb-4">
                    <label className="block mb-2 text-sm">Value</label>
                    <input
                        value={value}
                        onChange={(e) => setValue(e.target.value)}
                        className="w-full p-2 rounded bg-gray-800 border border-gray-700"
                    />
                </div>

                <div className="flex gap-3 mb-4">
                    <button onClick={handleUpsert} className="bg-blue-500 px-4 py-2 rounded">
                        Upsert
                    </button>
                    <button onClick={handleGet} className="bg-green-500 px-4 py-2 rounded">
                        Get
                    </button>
                </div>

                {result && (
                    <div className="p-3 bg-gray-800 rounded">
                        <p className="text-sm">{result}</p>
                    </div>
                )}
            </div>
        </div>
    );
}