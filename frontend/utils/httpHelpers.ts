/**
 * 处理 Fetch 响应，如果状态码不正常则解析并抛出错误信息
 * @param res Fetch API 返回的 Response 对象
 */
export async function handleResponseError(res: Response): Promise<void> {
    if (res.ok) return;

    let finalMsg: string;
    const rawBody = await res.text();

    try {
        // 尝试解析为 JSON
        const jsonBody = JSON.parse(rawBody);
        // 根据后端常见的字段名提取错误信息
        finalMsg = jsonBody.message || jsonBody.msg || jsonBody.error || rawBody;
    } catch {
        // 解析失败说明是纯文本 (如 "Permission denied")
        finalMsg = rawBody;
    }

    // 抛出错误，包含后端返回的具体信息或状态码
    throw new Error(finalMsg || `请求失败，状态码: ${res.status}`);
}