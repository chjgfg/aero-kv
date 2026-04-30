import { verifyBatchProof, verifySingleProof } from "./merkle";
import Swal from 'sweetalert2';

// 🔥 从 .env 环境变量读取（最标准企业级方案）
const API_BASE = process.env.NEXT_PUBLIC_API_URL!;

const health = async () => {
    const res = await fetch(`${API_BASE}/health`);
    const data = await res.json();
    return data;
}

const initAuth = async () => {
    const res = await fetch(`${API_BASE}/auth/init-admin`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        }
    });
    const data = await res.json();
    console.log(data);
    return data;
}

// const setAuth = async (new_admin: string) => {
//     const url = `${API_BASE}/auth/set-admin?new_admin=${new_admin}`;
//     const res = await fetch(url, {
//         method: "POST",
//         headers: {
//             "Content-Type": "application/json",
//         }
//     });
//     const data = await res.json();
//     console.log(data);
//     return data;
// }

const setPause = async (paused: boolean) => {
    const url = `${API_BASE}/auth/set-pause?paused=${paused}`;
    const res = await fetch(url, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        }
    });
    const data = await res.json();
    console.log(data);
    return data;
}

const initFee = async () => {
    const res = await fetch(`${API_BASE}/fee/init-fee`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        }
    });
    const data = await res.json();
    console.log(data);
    return data;
}

const setFee = async (base_fee: number, fee_per_byte: number, scan_fee_per_item: number) => {
    const res = await fetch(`${API_BASE}/fee/set-fee`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            base_fee: base_fee,
            fee_per_byte: fee_per_byte,
            scan_fee_per_item: scan_fee_per_item,
        }),
    });

    if (!res.ok) {
        throw new Error(`请求失败: ${res.status}`);
    }

    const data = await res.json();
    console.log(data);
    return data;
}

const initStorage = async () => {
    const res = await fetch(`${API_BASE}/kv/init-storage`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        }
    });
    const data = await res.json();
    console.log(data);
    return data;
}

const initCounter = async () => {
    const res = await fetch(`${API_BASE}/kv/init-counter`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        }
    });
    const data = await res.json();
    console.log(data);
    return data;
}

const upsert = async (key: string, value: string) => {
    const res = await fetch(`${API_BASE}/kv/upsert`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            key: key,
            value: value,
            limit: 0,
        }),
    });

    if (!res.ok) {
        throw new Error(`请求失败: ${res.status}`);
    }

    const data = await res.json();
    console.log(data);
    return data;
}

const deleted = async (key: string) => {
    const url = `${API_BASE}/kv/delete?key=${key}`;
    const res = await fetch(url, {
        method: "DELETE",
        headers: {
            "Content-Type": "application/json",
        }
    });
    const data = await res.json();
    console.log(data);
    return data;
}

const gets = async (key: string) => {
    try {
        const url = `${API_BASE}/kv/get?key=${key}`;
        const res = await fetch(url);

        if (!res.ok) {
            throw new Error(`HTTP 错误: ${res.status} ${res.statusText}`);
        }

        const data = await res.json();
        const isValid = await verifySingleProof(data);
        console.log(`✅ Merkle 验证结果: ${isValid ? '通过（数据未被篡改）' : '失败（数据被篡改）'}`);

        console.log("✅ 成功收到数据:", data);
        return data;
    } catch (err) {
        console.error("❌ 请求失败:", err);
    }
};

const scan = async (key: string, limit: number) => {
    const res = await fetch(`${API_BASE}/kv/scan`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            key: key,
            value: "",
            limit: limit,
        }),
    });

    if (!res.ok) {
        throw new Error(`请求失败: ${res.status}`);
    }

    const data = await res.json();
    const allValid = await verifyBatchProof(data);
    console.log(`✅ Merkle 验证结果: ${allValid ? '通过（数据未被篡改）' : '失败（数据被篡改）'}`);

    console.log(data);
    return data;
}

const page = async (page: number, limit: number) => {
    const res = await fetch(`${API_BASE}/kv/page`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            page: page,
            limit: limit,
        }),
    });

    console.log(res);
    if (res.status !== 200) {
        // throw new Error(`请求失败: ${res.status}`);
        Swal.fire({
            title: '请求错误',
            text: `请先一键初始化`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
        return;
    }

    const data = await res.json();
    const allValid = await verifyBatchProof(data);
    console.log(`✅ Merkle 验证结果: ${allValid ? '通过（数据未被篡改）' : '失败（数据被篡改）'}`);

    console.log(data);
    return data;
}

export {
    health, initAuth, setPause, initFee, setFee, initStorage, upsert, deleted, scan, gets, page, initCounter
}