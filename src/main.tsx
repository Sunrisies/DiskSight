import React from "react";
import ReactDOM from "react-dom/client";
import "./App.css";
import App from "./App";
import { invoke } from "@tauri-apps/api/core";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
// 这些语句可以复制粘贴到现有的代码下面，但不要替换整个文件！！

// 在 TypeScript 中实现的一个 sleep 函数
function sleep(seconds: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, seconds * 1000));
}

// 设置函数
async function setup() {
  // 模拟执行一个很重的前端设置任务
  console.log('Performing really heavy frontend setup task...')
  await sleep(1);
  console.log('Frontend setup task complete!')
  // 设置前端任务为完成
  invoke('set_complete', { task: 'frontend' })
}

// 实际上的 JavaScript main 函数
window.addEventListener("DOMContentLoaded", () => {
  setup()
});