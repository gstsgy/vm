// vm-gui：Tauri v1 后端，复用 vm-core 的全部逻辑。
// 所有命令返回 Result<T, String>，便于前端直接展示错误。

use vm_core::{self, Config};

type CmdResult<T> = Result<T, String>;

fn with_cfg<F, T>(f: F) -> CmdResult<T>
where
    F: FnOnce(&mut Config) -> vm_core::Result<T>,
{
    let mut cfg = vm_core::load().map_err(|e| e.to_string())?;
    let r = f(&mut cfg).map_err(|e| e.to_string())?;
    vm_core::save(&cfg).map_err(|e| e.to_string())?;
    Ok(r)
}

/// 返回完整配置快照（类目/版本/当前版本）。
#[tauri::command]
fn get_state() -> CmdResult<Config> {
    vm_core::load().map_err(|e| e.to_string())
}

#[tauri::command]
fn add_category(name: String, desc: String) -> CmdResult<()> {
    with_cfg(|cfg| vm_core::add_category(cfg, &name, &desc))?;
    // 创建 ~/.vm/<类目> 目录并加入用户 PATH
    vm_core::init_category_dir(&name).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn add_version(category: String, version: String, path: String, bin: Option<String>) -> CmdResult<()> {
    with_cfg(|cfg| vm_core::add_version(cfg, &category, &version, &path, bin))
}

#[tauri::command]
fn remove_category(name: String) -> CmdResult<()> {
    with_cfg(|cfg| vm_core::remove_category(cfg, &name))?;
    // 清理 ~/.vm/<类目> 链接/目录并移出用户 PATH
    vm_core::cleanup_category_dir(&name).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn remove_version(category: String, version: String) -> CmdResult<()> {
    with_cfg(|cfg| vm_core::remove_version(cfg, &category, &version))
}

/// 切换版本：更新配置并重建当前版本的 bin 链接。
#[tauri::command]
fn use_version(category: String, version: String) -> CmdResult<()> {
    let mut cfg = vm_core::load().map_err(|e| e.to_string())?;
    vm_core::use_version(&mut cfg, &category, &version).map_err(|e| e.to_string())?;
    vm_core::link_active_bin(&cfg, &category).map_err(|e| e.to_string())?;
    vm_core::save(&cfg).map_err(|e| e.to_string())
}

/// 生成需要加入 PATH 的环境片段（按当前类目）。
#[tauri::command]
fn env_snippet() -> CmdResult<String> {
    let cfg = vm_core::load().map_err(|e| e.to_string())?;
    let dir = vm_core::config_dir();
    let mut out = String::new();
    for c in vm_core::list_categories(&cfg) {
        let base = dir.join(c);
        out.push_str(&format!(
            "export PATH=\"{}:{}:$PATH\"\n",
            base.join("bin").display(),
            base.display()
        ));
    }
    Ok(out)
}

fn main() {
    let result = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_state,
            add_category,
            add_version,
            remove_category,
            remove_version,
            use_version,
            env_snippet
        ])
        .run(tauri::generate_context!());

    if let Err(e) = result {
        let msg = format!("vm-gui 启动失败: {e}\n\n常见原因：系统未安装 WebView2 Runtime。\n请下载 Edge WebView2 Runtime 后再试。");
        eprintln!("{msg}");
        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                MessageBoxW, MB_ICONERROR, MB_OK,
            };
            let mut wide: Vec<u16> = msg.encode_utf16().collect();
            wide.push(0);
            MessageBoxW(
                std::ptr::null_mut(),
                wide.as_ptr(),
                [0x76, 0x6D, 0x2D, 0x67, 0x75, 0x69, 0x00].as_ptr(), // "vm-gui\0"
                MB_OK | MB_ICONERROR,
            );
        }
        std::process::exit(1);
    }
}
