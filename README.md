# vm — 轻量级版本管理工具

使用 Rust 编写。支持**用户自定义类目**（如 nodejs / java），为每个类目添加**已安装版本**，
并通过符号链接（Unix）/ junction（Windows）实现**快速版本切换**。提供 **CLI** 与 **Tauri GUI** 两种操作方式。

## 架构

- `vm-core`：平台无关核心逻辑（配置、数据模型、切换），被 CLI 与 GUI 复用。
- `vm-cli`：基于 `clap` 的命令行（`vm` 可执行文件）。
- `vm-gui`：**Tauri v1** 应用 + **纯静态前端**（无前端框架，零构建步骤），复用 `vm-core`。

> 框架选型说明：GUI 采用 Tauri（Rust 后端 + 系统 WebView2），不引入 Electron/Angular 运行时，
> 内存占用低、安装包小；前端为原生 HTML/JS，构建产物极小。目标：Windows 安装包 < 10MB。

## 原理

每个类目维护 `~/.vm/<类目>/bin`：

- Unix 上为指向「当前版本 bin 目录」的 **symlink**
- Windows 上为 **junction**（无需管理员权限）

用户只需把 `~/.vm/<类目>/bin` 加入 `PATH` 一次（`vm init` 打印片段），之后 `vm use`
只重建这一个链接，新开终端立即生效。

## 快速开始（CLI）

```bash
cargo build --release -p vm-cli

# 新增类目（-d 说明可选）
vm addc nodejs -d "Node.js"

# 添加版本（path 为安装根目录；需先创建类目；Windows 上 Node 的 exe 在根目录可用 --bin 指定或自动探测）
vm add nodejs 20 /opt/node-v20
vm add nodejs 18 /opt/node-v18

# 切换（vm <类目> <版本>）
vm nodejs 20

# 查看当前版本（vm <类目>）
vm nodejs

# 列出版本（vm list [类目]）
vm list nodejs

# 删除版本 / 类目
vm remove nodejs 18
vm remove nodejs
```

## GUI

```bash
cd vm-gui/src-tauri
cargo tauri dev      # 开发预览
cargo tauri build    # 打包（Windows 产出 nsis 安装包 + 便携 exe）
```

GUI 通过 Tauri 命令调用 `vm-core`，功能与 CLI 一致：类目/版本的增删、版本切换、PATH 片段复制。

## 自动构建（CI）

`.github/workflows/build.yml` 在推送到 `main` / 打 `v*` 标签 / 手动触发时，于
`windows-latest` 上自动构建：

- CLI：`x86_64-pc-windows-msvc` 可执行文件 `vm.exe`
- GUI：Tauri 打包为 `nsis` 安装包 + 便携 `vm-gui.exe`
- **体积校验**：安装包与便携 exe 均不得超过 10MB，否则构建失败
- 打 `v*` 标签时自动创建 GitHub Release 并上传上述产物

## 测试

```bash
cargo test -p vm-core
```

## 重新生成图标

```bash
python3 scripts/make_icons.py   # 生成 vm-gui/src-tauri/icons/{icon.png,icon.ico}
```
