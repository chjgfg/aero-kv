import { upsert } from "./http";

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

    data.forEach(async (item: any) => {
        try {
            console.log('开始');
            console.log(item.key);
            console.log(item.value);
            await upsert(item.key, item.value);
            await sleep(5000);  // 等待 2s
            console.log('5000ms 后执行');
        } catch (e) {
            console.log(e)
        }
    });
}


export {
    moke
}