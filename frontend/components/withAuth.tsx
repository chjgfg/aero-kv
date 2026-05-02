"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";

export default function withAuth(Component: any) {
    return function ProtectedRoute(props: any) {
        const router = useRouter();
        const [isVerified, setIsVerified] = useState(false);

        useEffect(() => {
            const pubkey = localStorage.getItem("my_pubkey");
            if (!pubkey) {
                // 如果没登录，跳转到登录页
                router.replace("/login");
            } else {
                // 已登录，允许渲染
                setIsVerified(true);
            }
        }, [router]);

        // 在验证完成前，返回空内容或 Loading，防止页面内容闪现
        if (!isVerified) {
            return <div className="min-h-screen bg-[#0b0f1a]" />;
        }

        return <Component {...props} />;
    };
}