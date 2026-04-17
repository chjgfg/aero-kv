"use client";

import React, { useState } from 'react';
import { health, initAuth, setAuth, setPause, initFee, setFee, initStorage, upsert, deleted, scan, } from "../../utils/http.ts";

// 这里假设你把刚才的 http.ts 函数都放在了 @/lib/api 中，或者直接写在这个文件上方
// 为了演示方便，我这里直接引用你定义的函数名

export default function AdminDashboard() {
    const [logs, setLogs] = useState<string[]>([]);

    // 输入框状态管理
    const [adminAddress, setAdminAddress] = useState("");
    const [kvKey, setKvKey] = useState("");
    const [kvValue, setKvValue] = useState("");
    const [kvLimit, setKvLimit] = useState(10);
    const [feeBase, setFeeBase] = useState(1000);
    const [feePerByte, setFeePerByte] = useState(1);
    const [feeScan, setFeeScan] = useState(10);

    // 日志打印工具
    const addLog = (msg: string) => {
        setLogs(prev => [`[${new Date().toLocaleTimeString()}] ${msg}`, ...prev.slice(0, 19)]);
    };

    // 统一处理 API 调用
    const handleAction = async (name: string, apiFunc: () => Promise<any>) => {
        try {
            addLog(`正在执行: ${name}...`);
            const result = await apiFunc();
            addLog(`✅ ${name} 成功: ${JSON.stringify(result)}`);
        } catch (err: any) {
            addLog(`❌ ${name} 失败: ${err.message}`);
        }
    };

    return (
        <div style={{ padding: '20px', maxWidth: '1200px', margin: '0 auto', fontFamily: 'sans-serif' }}>
            <h1 style={{ borderBottom: '2px solid #333' }}>DiamondDB 管理控制台</h1>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>

                {/* 左侧：操作面板 */}
                <div>
                    {/* 初始化区域 */}
                    <section style={sectionStyle}>
                        <h3>🚀 系统初始化 (依次执行)</h3>
                        <div style={btnGroupStyle}>
                            <button onClick={() => handleAction("健康检查", health)}>Health Check</button>
                            <button onClick={() => handleAction("初始化权限", initAuth)} style={initBtnStyle}>Init Auth</button>
                            <button onClick={() => handleAction("初始化费用", initFee)} style={initBtnStyle}>Init Fee</button>
                            <button onClick={() => handleAction("初始化存储", initStorage)} style={initBtnStyle}>Init Storage</button>
                        </div>
                    </section>

                    {/* 权限管理 */}
                    <section style={sectionStyle}>
                        <h3>🔐 权限控制</h3>
                        <input
                            placeholder="新管理员钱包地址 (Base58)"
                            value={adminAddress}
                            onChange={(e) => setAdminAddress(e.target.value)}
                            style={inputStyle}
                        />
                        <div style={btnGroupStyle}>
                            <button onClick={() => handleAction("设置管理员", () => setAuth(adminAddress))}>更新管理员</button>
                            <button onClick={() => handleAction("暂停合约", () => setPause(true))} style={{ backgroundColor: '#ff4d4f' }}>紧急暂停</button>
                            <button onClick={() => handleAction("恢复合约", () => setPause(false))} style={{ backgroundColor: '#52c41a' }}>恢复运行</button>
                        </div>
                    </section>

                    {/* KV 存储操作 */}
                    <section style={sectionStyle}>
                        <h3>📦 KV 存储操作 (SkipList)</h3>
                        <div style={{ display: 'flex', gap: '10px', marginBottom: '10px' }}>
                            <input placeholder="Key" value={kvKey} onChange={(e) => setKvKey(e.target.value)} style={inputStyle} />
                            <input placeholder="Value" value={kvValue} onChange={(e) => setKvValue(e.target.value)} style={inputStyle} />
                            <input type="number" placeholder="Limit" value={kvLimit} onChange={(e) => setKvLimit(Number(e.target.value))} style={{ ...inputStyle, width: '80px' }} />
                        </div>
                        <div style={btnGroupStyle}>
                            <button onClick={() => handleAction("写入(Upsert)", () => upsert(kvKey, kvValue, kvLimit))}>写入数据</button>
                            <button onClick={() => handleAction("查询(Get)", () => gets(kvKey))}>单点查询</button>
                            <button onClick={() => handleAction("范围扫描(Scan)", () => scan(kvKey, kvLimit))}>范围扫描</button>
                            <button onClick={() => handleAction("删除(Delete)", () => deleted(kvKey))} style={{ backgroundColor: '#ff4d4f' }}>删除Key</button>
                        </div>
                    </section>

                    {/* 手续费设置 */}
                    <section style={sectionStyle}>
                        <h3>💰 手续费配置</h3>
                        <div style={{ display: 'flex', gap: '10px', marginBottom: '10px' }}>
                            <label>基础: <input type="number" value={feeBase} onChange={(e) => setFeeBase(Number(e.target.value))} style={inputStyle} /></label>
                            <label>字节: <input type="number" value={feePerByte} onChange={(e) => setFeePerByte(Number(e.target.value))} style={inputStyle} /></label>
                            <label>扫描: <input type="number" value={feeScan} onChange={(e) => setFeeScan(Number(e.target.value))} style={inputStyle} /></label>
                        </div>
                        <button onClick={() => handleAction("更新费用", () => setFee(feeBase, feePerByte, feeScan))}>保存费用配置</button>
                    </section>
                </div>

                {/* 右侧：实时日志 */}
                <div>
                    <section style={{ ...sectionStyle, height: '100%', backgroundColor: '#1e1e1e', color: '#00ff00' }}>
                        <h3>📡 实时运行日志</h3>
                        <div style={{ fontSize: '12px', lineHeight: '1.6', fontFamily: 'monospace' }}>
                            {logs.map((log, i) => <div key={i} style={{ borderBottom: '1px solid #333', padding: '4px 0' }}>{log}</div>)}
                        </div>
                    </section>
                </div>

            </div>
        </div>
    );
}

// --- 样式定义 ---
const sectionStyle: React.CSSProperties = {
    border: '1px solid #ddd',
    padding: '15px',
    borderRadius: '8px',
    marginBottom: '20px',
    backgroundColor: '#f9f9f9'
};

const inputStyle: React.CSSProperties = {
    padding: '8px',
    borderRadius: '4px',
    border: '1px solid #ccc',
    width: '100%',
    marginBottom: '10px',
    color: '#333'
};

const btnGroupStyle: React.CSSProperties = {
    display: 'flex',
    gap: '10px',
    flexWrap: 'wrap'
};

const initBtnStyle: React.CSSProperties = {
    backgroundColor: '#1890ff',
    color: 'white'
};