//! 用户级 PATH 环境变量管理。
//! Windows：写入注册表 `HKCU\Environment`（无需管理员），并广播变更通知；
//! 类 Unix：持久化交由 shell profile（`vm init` 输出片段），此处为 no-op。

use std::path::Path;

use crate::Result;

/// 将目录追加到用户级 PATH（已存在则跳过）。
#[cfg(windows)]
pub fn add_to_user_path(dir: &Path) -> Result<()> {
    let dir_s = dir.display().to_string();
    let current = read_user_path()?;
    let exists = current
        .split(';')
        .any(|p| p.trim().trim_matches('"').eq_ignore_ascii_case(&dir_s));
    if !exists {
        let new = if current.trim().is_empty() {
            dir_s
        } else {
            format!("{};{}", current.trim_end_matches(';'), dir_s)
        };
        write_user_path(&new)?;
        broadcast_env_change();
    }
    Ok(())
}

/// 从用户级 PATH 中移除目录（不存在则跳过）。
#[cfg(windows)]
pub fn remove_from_user_path(dir: &Path) -> Result<()> {
    let dir_s = dir.display().to_string();
    let current = read_user_path()?;
    let parts: Vec<&str> = current
        .split(';')
        .filter(|p| {
            let t = p.trim().trim_matches('"');
            !t.is_empty() && !t.eq_ignore_ascii_case(&dir_s)
        })
        .collect();
    let new = parts.join(";");
    if new != current {
        write_user_path(&new)?;
        broadcast_env_change();
    }
    Ok(())
}

#[cfg(windows)]
fn read_user_path() -> Result<String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.open_subkey("Environment") {
        Ok(env) => Ok(env.get_value::<String, _>("Path").unwrap_or_default()),
        Err(_) => Ok(String::new()),
    }
}

#[cfg(windows)]
fn write_user_path(value: &str) -> Result<()> {
    use winreg::enums::{HKEY_CURRENT_USER, REG_EXPAND_SZ};
    use winreg::{RegKey, RegValue};
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (env, _) = hkcu.create_subkey("Environment")?;
    // 写 REG_EXPAND_SZ，保留 PATH 中 %VAR% 的展开能力
    let mut bytes: Vec<u8> = Vec::with_capacity((value.len() + 1) * 2);
    for u in value.encode_utf16().chain(std::iter::once(0u16)) {
        bytes.extend_from_slice(&u.to_le_bytes());
    }
    env.set_raw_value(
        "Path",
        &RegValue {
            bytes,
            vtype: REG_EXPAND_SZ,
        },
    )?;
    Ok(())
}

/// 广播 WM_SETTINGCHANGE，让资源管理器/新开终端感知 PATH 变更。
#[cfg(windows)]
fn broadcast_env_change() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };
    let s: Vec<u16> = "Environment\0".encode_utf16().collect();
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            s.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            3000,
            std::ptr::null_mut(),
        );
    }
}

#[cfg(not(windows))]
pub fn add_to_user_path(_dir: &Path) -> Result<()> {
    Ok(())
}

#[cfg(not(windows))]
pub fn remove_from_user_path(_dir: &Path) -> Result<()> {
    Ok(())
}
