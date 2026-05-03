import { verifyBatchProof, verifySingleProof } from "./merkle";
import Swal from 'sweetalert2';
import { toast } from "sonner";
import { handleResponseError } from "./httpHelpers";

// 🔥 从 .env 环境变量读取（最标准企业级方案）
const API_BASE = process.env.NEXT_PUBLIC_API_URL!;

type Action =
    | "InitAuth" | "InitStorage" | "InitCounter" | "InitFee"
    | "RaftUpsert" | "RaftDelete" | "Get" | "Scan"
    | "Page" | "RaftPause" | "RaftFee";

const health = async () => {
    const res = await fetch(`${API_BASE}/health`);
    const data = await res.json();
    return data;
}

// --------------------------------------------------------------------------------------------------
const initAuth = async () => {
    try {
        const savedPubkey = localStorage.getItem("my_pubkey");
        const res = await fetch(`${API_BASE}/auth/init-admin`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                // 🌟 在这里塞进 Header
                "x-user-pubkey": savedPubkey || ""
            }
        });
        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);
        const data = await res.json();
        console.log(data);
        return data;
    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '初始化权限出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
    }
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
    try {
        const savedPubkey = localStorage.getItem("my_pubkey");
        const url = `${API_BASE}/auth/set-pause?paused=${paused}`;
        const res = await fetch(url, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                // 🌟 在这里塞进 Header
                "x-user-pubkey": savedPubkey || ""
            }
        });
        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);
        const data = await res.json();
        console.log(data);
        return data;
    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '设置全局开关出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
    }
}

const login = async (pubkey: string) => {
    try {
        const res = await fetch(`${API_BASE}/auth/login`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            // 将公钥字符串传给后端
            body: JSON.stringify({
                user_pubkey: pubkey
            }),
        });
        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);
        const data = await res.json();
        console.log(data);
        // console.log(res);
        if (res.ok) {
            // 🌟 登录成功，把公钥存起来
            localStorage.setItem("my_pubkey", pubkey);
            localStorage.setItem("is_admin", data.is_admin);
            localStorage.setItem("permissions", data.permissions);
        }
        return data;
    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '登录出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
    }
}

const logout = async () => {
    try {
        const savedPubkey = localStorage.getItem("my_pubkey");
        const res = await fetch(`${API_BASE}/auth/logout`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            // 将公钥字符串传给后端
            body: JSON.stringify({
                user_pubkey: savedPubkey
            }),
        });
        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);
        const data = await res.json();
        console.log(data);
        console.log(res);
        if (res.ok) {
            // 🌟 登录成功，把公钥存起来
            localStorage.removeItem("my_pubkey");
            localStorage.removeItem("is_admin"); // 如果你存了管理员标识
            localStorage.removeItem("permissions"); // 如果你存了管理员标识
        }
        return data;
    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '登出出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
    }
}

const grant = async (pubkey: string, selectedActions: Action[]) => {
    try {
        const savedPubkey = localStorage.getItem("my_pubkey");
        const res = await fetch(`${API_BASE}/auth/grant`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            // 将公钥字符串传给后端
            body: JSON.stringify({
                admin_pubkey: savedPubkey,
                user_pubkey: pubkey,
                perm_char: selectedActions,
            }),
        });
        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);
        const data = await res.json();
        toast.success("操作成功", {
            description: `给公钥为 ${pubkey} 的用户授权成功`,
            position: "bottom-right", // 定位在右下角
        });
        console.log("结果打印", data);
        return data;
    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '授权出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
    }
}

const revoke = async (pubkey: string,) => {
    try {
        const savedPubkey = localStorage.getItem("my_pubkey");
        const res = await fetch(`${API_BASE}/auth/revoke`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            // 将公钥字符串传给后端
            body: JSON.stringify({
                admin_pubkey: savedPubkey,
                user_pubkey: pubkey,
            }),
        });
        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);
        const data = await res.json();
        toast.success("操作成功", {
            description: `收回公钥为 ${pubkey} 的用户权限成功`,
            position: "bottom-right", // 定位在右下角
        });
        console.log("结果打印", data);
        return data;
    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '收回权限出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
    }
}

const admin_page = async (page: number, limit: number) => {
    try {
        const res = await fetch(`${API_BASE}/auth/page`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                page: page,
                limit: limit,
            }),
        });
        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);
        const data = await res.json();
        console.log("结果打印", data);
        return data;
    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '权限分页查询出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
    }
}

const admin_get = async (key: string, limit: number) => {
    try {
        const res = await fetch(`${API_BASE}/auth/get`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                pubkey: key,
                limit: limit,
            }),
        });
        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);
        const data = await res.json();
        console.log("结果打印", data);
        return data;
    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '查询权限出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
    }
}

// --------------------------------------------------------------------------------------------------
const initFee = async () => {
    try {
        const savedPubkey = localStorage.getItem("my_pubkey");
        const res = await fetch(`${API_BASE}/fee/init-fee`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                // 🌟 在这里塞进 Header
                "x-user-pubkey": savedPubkey || ""
            }
        });
        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);
        const data = await res.json();
        console.log(data);
        return data;
    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '初始化手续费出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
    }
}

const setFee = async (base_fee: number, fee_per_byte: number, scan_fee_per_item: number): Promise<any> => {
    try {
        const savedPubkey = localStorage.getItem("my_pubkey");
        const res = await fetch(`${API_BASE}/fee/set-fee`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                // 🌟 在这里塞进 Header
                "x-user-pubkey": savedPubkey || ""
            },
            body: JSON.stringify({
                base_fee: base_fee,
                fee_per_byte: fee_per_byte,
                scan_fee_per_item: scan_fee_per_item,
            }),
        });

        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);

        const data = await res.json();
        console.log(res);
        console.log(data);
        return data;

    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '设置手续费出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
    }
}

// --------------------------------------------------------------------------------------------------
const initStorage = async () => {
    try {
        const savedPubkey = localStorage.getItem("my_pubkey");
        const res = await fetch(`${API_BASE}/kv/init-storage`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                // 🌟 在这里塞进 Header
                "x-user-pubkey": savedPubkey || ""
            }
        });
        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);
        const data = await res.json();
        console.log(data);
        return data;
    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '初始化存储出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
    }
}

const initCounter = async () => {
    try {
        const savedPubkey = localStorage.getItem("my_pubkey");
        const res = await fetch(`${API_BASE}/kv/init-counter`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                // 🌟 在这里塞进 Header
                "x-user-pubkey": savedPubkey || ""
            }
        });
        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);
        const data = await res.json();
        console.log(data);
        return data;
    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '初始化计数器出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
    }
}

const upsert = async (key: string, value: string) => {
    try {
        const savedPubkey = localStorage.getItem("my_pubkey");
        const res = await fetch(`${API_BASE}/kv/upsert`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                // 🌟 在这里塞进 Header
                "x-user-pubkey": savedPubkey || ""
            },
            body: JSON.stringify({
                key: key,
                value: value,
                limit: 0,
            }),
        });

        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);

        toast.success("操作成功", {
            description: `已成功插入键值为 ${key} - ${value} 数据, 等待页面自动刷新`,
            position: "bottom-right", // 定位在右下角
        });
        const data = await res.json();
        console.log(data);
        return data;
    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '更新数据出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
        // 🌟 必须加上这一行！抛出错误，外层才能捕获到
        throw err;
    }
}

const deleted = async (key: string) => {
    try {
        const savedPubkey = localStorage.getItem("my_pubkey");
        const url = `${API_BASE}/kv/delete?key=${key}`;
        const res = await fetch(url, {
            method: "DELETE",
            headers: {
                "Content-Type": "application/json",
                // 🌟 在这里塞进 Header
                "x-user-pubkey": savedPubkey || ""
            }
        });
        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);

        const data = await res.json();
        toast.success("操作成功", {
            description: `已成功删除键为 ${key} 的数据, 等待页面自动刷新`,
            position: "bottom-right", // 定位在右下角
        });
        console.log(data);
        return data;
    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '删除数据出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
    }
}

const gets = async (key: string) => {
    try {
        const savedPubkey = localStorage.getItem("my_pubkey");
        const url = `${API_BASE}/kv/get?key=${key}`;
        const res = await fetch(url, {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
                // 🌟 在这里塞进 Header
                "x-user-pubkey": savedPubkey || ""
            }
        });

        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);

        const data = await res.json();
        const isValid = await verifySingleProof(data);
        console.log(`✅ Merkle 验证结果: ${isValid ? '通过（数据未被篡改）' : '失败（数据被篡改）'}`);
        toast.success("操作成功", {
            description: `查找键为 ${key} 的数据成功`,
            position: "bottom-right", // 定位在右下角
        });
        console.log("✅ 成功收到数据:", data);
        return data;
    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '查找出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
    }
};

const scan = async (key: string, limit: number) => {
    try {
        const savedPubkey = localStorage.getItem("my_pubkey");
        const res = await fetch(`${API_BASE}/kv/scan`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                // 🌟 在这里塞进 Header
                "x-user-pubkey": savedPubkey || ""
            },
            body: JSON.stringify({
                key: key,
                value: "",
                limit: limit,
            }),
        });

        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);

        const data = await res.json();
        const allValid = await verifyBatchProof(data);
        console.log(`✅ Merkle 验证结果: ${allValid ? '通过（数据未被篡改）' : '失败（数据被篡改）'}`);
        toast.success("操作成功", {
            description: `扫描前缀键为 ${key} 的数据成功, 共查出 ${data.pairs.length} 条`,
            position: "bottom-right", // 定位在右下角
        });
        console.log(data);
        return data;
    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '扫描出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
    }
}

const page = async (page: number, limit: number) => {
    try {
        const savedPubkey = localStorage.getItem("my_pubkey");
        const res = await fetch(`${API_BASE}/kv/page`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                // 🌟 在这里塞进 Header
                "x-user-pubkey": savedPubkey || ""
            },
            body: JSON.stringify({
                page: page,
                limit: limit,
            }),
        });
        // 🌟 使用封装好的方法：如果报错会直接抛出并进入 catch
        await handleResponseError(res);

        const data = await res.json();
        const allValid = await verifyBatchProof(data);
        console.log(`✅ Merkle 验证结果: ${allValid ? '通过（数据未被篡改）' : '失败（数据被篡改）'}`);
        toast.success("操作成功", {
            description: `分页查询数据成功, 总共查到 ${data.pairs.length} 条数据`,
            position: "bottom-right", // 定位在右下角
        });
        console.log(data);
        return data;
    } catch (err: any) {
        console.error("❌ 请求失败:", err);
        Swal.fire({
            title: '分页查找出错',
            text: `${err.message}`,
            icon: 'error',
            confirmButtonText: '知道了'
        });
    }
}

// --------------------------------------------------------------------------------------------------
export {
    health, initAuth, setPause, initFee, setFee, initStorage, upsert, deleted, scan, gets, page, initCounter, login, logout, grant, revoke, admin_get, admin_page
};
export type { Action };
