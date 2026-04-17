const health = async () => {
    const res = await fetch("http://192.168.40.131/health");
    const data = await res.json();
    return data;
}

// ----------------------------------------------------------------------------------------------------------------------------------

const initAuth = async () => {
    const res = await fetch("http://192.168.40.131/auth/init-admin", {
        method: "POST", // 后端路由是 POST
        headers: {
            "Content-Type": "application/json",
        }
    });
    const data = await res.json();
    return data;
}

const setAuth = async (new_admin: string) => {
    // 将参数拼接到 URL 后面
    const url = `http://192.168.40.131/auth/set-admin?new_admin=${new_admin}`;
    const res = await fetch(url, {
        method: "POST", // 后端路由是 POST
        headers: {
            "Content-Type": "application/json",
        }
    });
    const data = await res.json();
    return data;
}

const setPause = async (paused: boolean) => {
    // 将参数拼接到 URL 后面
    const url = `http://192.168.40.131/auth/set-pause?new_admin=${paused}`;
    const res = await fetch(url, {
        method: "POST", // 后端路由是 POST
        headers: {
            "Content-Type": "application/json",
        }
    });
    const data = await res.json();
    return data;
}

// ----------------------------------------------------------------------------------------------------------------------------------

const initFee = async () => {
    const res = await fetch("http://192.168.40.131/fee/init-fee", {
        method: "POST", // 后端路由是 POST
        headers: {
            "Content-Type": "application/json",
        }
    });
    const data = await res.json();
    return data;
}

const setFee = async (base_fee: number, fee_per_byte: number, scan_fee_per_item: number) => {
    const res = await fetch("http://192.168.40.131/fee/set-fee", {
        method: "POST", // 必须是 POST，因为你的后端路由是这么定义的
        headers: {
            "Content-Type": "application/json",
        },
        // 2. 将参数放入 Body 传给后端
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
    return data;
}

// ----------------------------------------------------------------------------------------------------------------------------------

const initStorage = async () => {
    const res = await fetch("http://192.168.40.131/kv/init-storage", {
        method: "POST", // 后端路由是 POST
        headers: {
            "Content-Type": "application/json",
        }
    });
    const data = await res.json();
    return data;
}

const upsert = async (key: string, value: string, limit: number) => {
    const res = await fetch("http://192.168.40.131/kv/upsert", {
        method: "POST", // 必须是 POST，因为你的后端路由是这么定义的
        headers: {
            "Content-Type": "application/json",
        },
        // 2. 将参数放入 Body 传给后端
        body: JSON.stringify({
            key: key,
            value: value,
            limit: limit,
        }),
    });

    if (!res.ok) {
        throw new Error(`请求失败: ${res.status}`);
    }

    const data = await res.json();
    return data;
}

const deleted = async (key: string) => {
    // 将参数拼接到 URL 后面
    const url = `http://192.168.40.131/kv/delete?key=${key}`;
    const res = await fetch(url, {
        method: "DELETE", // 后端路由是 POST
        headers: {
            "Content-Type": "application/json",
        }
    });
    const data = await res.json();
    return data;
}

const gets = async (key: string) => {
    // 将参数拼接到 URL 后面
    const url = `http://192.168.40.131/kv/get?key=${key}`;
    const res = await fetch(url);
    const data = await res.json();
    return data;
}

const scan = async (key: string, limit: number) => {
    const res = await fetch("http://192.168.40.131/kv/scan", {
        method: "POST", // 必须是 POST，因为你的后端路由是这么定义的
        headers: {
            "Content-Type": "application/json",
        },
        // 2. 将参数放入 Body 传给后端
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
    return data;
}

// ----------------------------------------------------------------------------------------------------------------------------------

export {
    health, initAuth, setAuth, setPause, initFee, setFee, initStorage, upsert, deleted, scan, gets
}