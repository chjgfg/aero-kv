import { upsert } from "./http";
import Swal from 'sweetalert2';

const sleep = async (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

const moke = async () => {
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
            "key": "aaaaaa",
            "value": "a5",
        },
        {
            "key": "aaaaaaa",
            "value": "a6",
        },
        {
            "key": "aaaaaaaa",
            "value": "a7",
        },
        {
            "key": "aaaaaaaaa",
            "value": "a8",
        },
        {
            "key": "aaaaaaaaaa",
            "value": "a9",
        },
        {
            "key": "aaaaaaaaaa",
            "value": "a10",
        },
        {
            "key": "aaaaaaaaaaa",
            "value": "a11",
        },
        {
            "key": "aaaaaaaaaaaaa",
            "value": "a12",
        },
        {
            "key": "aaaaaaaaaaaaaa",
            "value": "a13",
        },
        {
            "key": "aaaaaaaaaaaaaaa",
            "value": "a14",
        },
        {
            "key": "aaaaaaaaaaaaaaaa",
            "value": "a15",
        },
        {
            "key": "aaaaaaaaaaaaaaaaa",
            "value": "a16",
        },
        {
            "key": "aaaaaaaaaaaaaaaaaa",
            "value": "a17",
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
    ];

    // 💡 使用 for...of 才能真正让 await 按顺序执行
    for (const item of data) {
        try {
            console.log(item);
            await upsert(item.key, item.value);
            // 注意：你注释写等待 2s，但代码是 2000 ms
            await sleep(2000);
            console.log('2000 ms 后执行');
        } catch (e) {
            // 🛑 第一次报错就在这里捕获
            // console.error("遇到错误，停止后续操作:", e);
            Swal.fire({
                title: '写入出错',
                text: `在处理 ${item.key} 时发生错误`,
                icon: 'error',
                confirmButtonText: '知道了'
            });
            break;
        }
    }
}


export {
    moke
}