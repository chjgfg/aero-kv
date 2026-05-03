"use client";

import { ConnectionProvider, WalletProvider } from "@solana/wallet-adapter-react";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import { clusterApiUrl } from "@solana/web3.js";
import "@solana/wallet-adapter-react-ui/styles.css";
import "./globals.css";
import { Toaster } from 'sonner'; // 如果你用的是 sonner

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const network = "devnet";
  const endpoint = clusterApiUrl(network);
  const wallets: any = []; // 自动识别 Phantom，无需手动实例化

  return (
    <html lang="zh-CN">
      <body>
        <ConnectionProvider endpoint={endpoint}>
          <WalletProvider wallets={wallets} autoConnect>
            <WalletModalProvider>{children}</WalletModalProvider>
          </WalletProvider>
        </ConnectionProvider>
        {/* 🌟 必须加上这个组件，提示才会显示出来 */}
        <Toaster
          position="bottom-right"
          theme="dark"
          toastOptions={{
            style: {
              background: '#1e293b', // 匹配你面板的深蓝色 (slate-800)
              color: '#f1f5f9',      // 亮灰色文字
              border: '1px solid #4f46e5', // 靛蓝色边框 (indigo-600)
            },
            className: 'my-custom-toast',
          }}
        />
      </body>
    </html>
  );
}