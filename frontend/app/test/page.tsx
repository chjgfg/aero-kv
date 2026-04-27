
"use client";

import React, { useState, useEffect } from 'react';
import { health, initAuth, setAuth, setPause, initFee, setFee, initStorage, upsert, deleted, scan, gets, page } from "../../utils/http.ts";

export default function AdminDashboard() {
    const [logs, setLogs] = useState<string[]>([]);
    const [adminAddress, setAdminAddress] = useState("");
    const [kvKey, setKvKey] = useState("");
    const [kvValue, setKvValue] = useState("");
    const [kvOffset, setKvOffset] = useState(0);
    const [kvLimit, setKvLimit] = useState(10);
    const [feeBase, setFeeBase] = useState(1000);
    const [feePerByte, setFeePerByte] = useState(1);
    const [feeScan, setFeeScan] = useState(10);

    // 日志打印
    const addLog = (msg: string) => {
        setLogs(prev => [`[${new Date().toLocaleTimeString()}] ${msg}`, ...prev.slice(0, 19)]);
    };

    // 统一 API 调用
    const handleAction = async (name: string, apiFunc: () => Promise<any>) => {
        try {
            addLog(`正在执行: ${name}...`);
            const result = await apiFunc();
            addLog(`✅ ${name} 成功: ${JSON.stringify(result)}`);
            return true;
        } catch (err: any) {
            addLog(`❌ ${name} 失败: ${err.message}`);
            return false;
        }
    };

    // ==============================================
    // 🔥 页面加载自动执行：initAuth → initFee → initStorage
    // ==============================================
    useEffect(() => {
        const autoInit = async () => {
            addLog("📦 页面加载完成，开始自动初始化...");
            
            // 按顺序执行，必须前一个成功才执行下一个
            const authOk = await handleAction("初始化权限", initAuth);
            // if (!authOk) return;

            const feeOk = await handleAction("初始化费用", initFee);
            // if (!feeOk) return;

            const storageOk = await handleAction("初始化存储", initStorage);
            // if (storageOk) {
            //     addLog("✅ 所有初始化任务执行完成！");
            // }
        };

        // 进入页面立即执行一次
        autoInit();
    }, []); // 空依赖 = 只执行一次

    return (
        <div style={{ padding: '20px', maxWidth: '1200px', margin: '0 auto', fontFamily: 'sans-serif' }}>
            <h1 style={{ borderBottom: '2px solid #333' }}>DiamondDB 管理控制台</h1>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
                <div>
                    <section style={sectionStyle}>
                        <h3>🚀 系统初始化 (依次执行)</h3>
                        <div style={btnGroupStyle}>
                            <button onClick={() => handleAction("健康检查", health)}>Health Check</button>
                            <button onClick={() => handleAction("初始化权限", initAuth)} style={initBtnStyle}>Init Auth</button>
                            <button onClick={() => handleAction("初始化费用", initFee)} style={initBtnStyle}>Init Fee</button>
                            <button onClick={() => handleAction("初始化存储", initStorage)} style={initBtnStyle}>Init Storage</button>
                        </div>
                    </section>

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

                    <section style={sectionStyle}>
                        <h3>📦 KV 存储操作 (SkipList + 分页)</h3>
                        <div style={{ display: 'flex', gap: '10px', marginBottom: '10px' }}>
                            <input placeholder="Key" value={kvKey} onChange={(e) => setKvKey(e.target.value)} style={inputStyle} />
                            <input placeholder="Value" value={kvValue} onChange={(e) => setKvValue(e.target.value)} style={inputStyle} />
                        </div>
                        {/* 分页：offset + limit */}
                        <div style={{ display: 'flex', gap: '10px', marginBottom: '10px' }}>
                            <input
                                type="number"
                                placeholder="Offset 起始位置"
                                value={kvOffset}
                                onChange={(e) => setKvOffset(Number(e.target.value))}
                                style={{ ...inputStyle, width: '120px' }}
                            />
                            <input
                                type="number"
                                placeholder="Limit 条数"
                                value={kvLimit}
                                onChange={(e) => setKvLimit(Number(e.target.value))}
                                style={{ ...inputStyle, width: '120px' }}
                            />
                        </div>
                        <div style={btnGroupStyle}>
                            <button onClick={() => handleAction("写入(Upsert)", () => upsert(kvKey, kvValue))}>写入数据</button>
                            <button onClick={() => handleAction("查询(Get)", () => gets(kvKey))}>单点查询</button>
                            <button onClick={() => handleAction("范围扫描(Scan)", () => scan(kvKey, kvOffset))}>范围扫描</button>
                            <button onClick={() => handleAction("分页(Page)", () => page(kvOffset, kvLimit))}>分页</button>
                            <button onClick={() => handleAction("删除(Delete)", () => deleted(kvKey))} style={{ backgroundColor: '#ff4d4f' }}>删除Key</button>
                        </div>
                    </section>

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