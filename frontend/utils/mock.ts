import { upsert } from "./http";
import Swal from 'sweetalert2';
import { toast } from "sonner";

const sleep = async (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

const mock = async () => {
    let data: any[] = [
        {
            "key": "a",
            "value": "a1",
        },
        {
            "key": "aa",
            "value": "a2",
        },
        {
            "key": "aaa",
            "value": "a3",
        },
        {
            "key": "aaaa",
            "value": "a4",
        },
        {
            "key": "aaaaa",
            "value": "a5",
        },
        {
            "key": "aaaaaa",
            "value": "a6",
        },
        {
            "key": "aaaaaaa",
            "value": "a7",
        },
        {
            "key": "aaaaaaaa",
            "value": "a8",
        },
        {
            "key": "aaaaaaaaa",
            "value": "a9",
        },
        {
            "key": "aaaaaaaaa",
            "value": "a10",
        },
        {
            "key": "aaaaaaaaaa",
            "value": "a11",
        },
        {
            "key": "aaaaaaaaaaaa",
            "value": "a12",
        },
        {
            "key": "aaaaaaaaaaaaa",
            "value": "a13",
        },
        {
            "key": "aaaaaaaaaaaaaa",
            "value": "a14",
        },
        {
            "key": "aaaaaaaaaaaaaaa",
            "value": "a15",
        },
        {
            "key": "aaaaaaaaaaaaaaaa",
            "value": "a16",
        },
        {
            "key": "aaaaaaaaaaaaaaaaa",
            "value": "a17",
        },
        {
            "key": "aaaaaaaaaaaaaaaaaa",
            "value": "a18",
        },
        {
            "key": "s",
            "value": "s",
        },
        {
            "key": "d",
            "value": "ad17",
        },
        {
            "key": "f",
            "value": "f",
        },
        {
            "key": "g",
            "value": "ggfff",
        },
        {
            "key": "h",
            "value": "hhhhh",
        },
        {
            "key": "j",
            "value": "ggjfff",
        },
        {
            "key": "k",
            "value": "kkkdkdk",
        },
        {
            "key": "l",
            "value": "ll",
        },
        {
            "key": "z1",
            "value": "z2",
        },
    ];

    // 💡 使用 for...of 才能真正让 await 按顺序执行
    for (const item of data) {
        try {
            console.log(item);
            await upsert(item.key, item.value);
            // toast.success("操作成功", {
            //     description: `已成功插入 ${item.key}`,
            //     position: "bottom-right", // 定位在右下角
            // });
        } catch (e) {
            // 🛑 第一次报错就在这里捕获
            // console.error("遇到错误，停止后续操作:", e);
            Swal.fire({
                title: '写入出错',
                text: `在处理 ${item.key} 时发生错误`,
                icon: 'error',
                confirmButtonText: '知道了'
            });
            return;
        }
    }
    Swal.fire({
        title: '写入完成',
        text: `已成功插入 ${data.length}条数据, 请手动刷新全表`,
        icon: 'success', // 核心修改：图标改为 success
        confirmButtonText: '太棒了'
    });
}


export {
    mock
}