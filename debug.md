# 调试步骤
启动 npm run dev
等待 Vite 在 http://localhost:1420 就绪
执行 cargo build --manifest-path src-tauri/Cargo.toml --bin HealthMonitor
启动 launch.json 里的 Rust 调试进程

图形界面：在vscode -> debug
如果已经使用 pnpm tauri dev 启动后台程序
选 Tauri Rust: Attach to HealthMonitor   然后找到 HealthMonitor.exe
如果未启动 后台exe：
Tauri Rust: Launch HealthMonitor

如果没有启动前台（npm run dev）
使用Tauri Rust: One-click full debug进行调试

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Tauri Rust: One-click full debug",
      "type": "cppvsdbg",
      "request": "launch",
      "program": "${workspaceFolder}/src-tauri/target/debug/HealthMonitor.exe",
      "cwd": "${workspaceFolder}/src-tauri",
      "preLaunchTask": "tauri: prepare full debug",
      "stopAtEntry": false,
      "console": "integratedTerminal"
    },
    {
      "name": "Tauri Rust: Launch HealthMonitor",
      "type": "cppvsdbg",
      "request": "launch",
      "program": "${workspaceFolder}/src-tauri/target/debug/HealthMonitor.exe",
      "cwd": "${workspaceFolder}/src-tauri",
      "preLaunchTask": "cargo: build tauri debug",
      "stopAtEntry": false,
      "console": "integratedTerminal"
    },
    {
      "name": "Tauri Rust: Attach to HealthMonitor",
      "type": "cppvsdbg",
      "request": "attach",
      "processId": "${command:pickProcess}"
    }
  ]
}

```

# 查看端口占用
如果报 1420 端口已占用，查找 id，然后kill掉再执行
```shell
Get-Process -Id (Get-NetTCPConnection -LocalPort 1420 -State Listen).OwningProcess
Stop-Process -Id 16604
```