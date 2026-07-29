# vm — 轻量级版本管理工具

使用 Rust 编写。支持**用户自定义类目**（如 nodejs / java），为每个类目添加**已安装版本**，
并通过符号链接（Unix）/ junction（Windows）实现**快速版本切换**。提供 CLI 与 Tauri+Angular GUI。

## 架构

- `vm-core`：平台无关核心逻辑（配置、数据模型、切换），被 CLI 与 GUI 复用。
- `vm-cli`：基于 `clap` 的命令行（`vm` 可执行文件）。
- `vm-gui`：Tauri 应用 + Angular 前端（M4 阶段实现）。

## 原理

每个类目维护 `~/.vm/<类目>/bin`：

- Unix 上为指向「当前版本 bin 目录」的 **symlink**
- Windows 上为 **junction**（无需管理员权限）

用户只需把 `~/.vm/<类目>/bin` 加入 `PATH` 一次（`vm init` 打印片段），之后 `vm use`
只重建这一个链接，新开终端立即生效。

## 快速开始（CLI）

```bash
cargo build --release -p vm-cli
# 新增类目
vm category add nodejs --desc "Node.js"
# 添加版本（path 为安装根目录；Windows 上 Node 的 exe 在根目录可用 --bin 指定或自动探测）
vm version add nodejs 20 /opt/node-v20
vm version add nodejs 18 /opt/node-v18
# 切换
vm use nodejs 20
# 查看
vm current nodejs
vm version list nodejs
```

## 测试

```bash
cargo test -p vm-core
```
