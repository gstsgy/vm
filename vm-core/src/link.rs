use std::path::Path;

use crate::Result;

/// 删除链接（symlink / junction）或空目录；路径不存在则直接跳过。
/// 用 `symlink_metadata` 只看路径本身，不解析目标，避免误删链接指向的内容。
pub fn remove_link(path: &Path) -> Result<()> {
    let meta = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    let ft = meta.file_type();
    if ft.is_symlink() {
        #[cfg(windows)]
        {
            // Windows 目录符号链接需 remove_dir；文件符号链接用 remove_file
            if std::fs::remove_dir(path).is_err() {
                std::fs::remove_file(path)?;
            }
        }
        #[cfg(not(windows))]
        std::fs::remove_file(path)?;
    } else if ft.is_dir() {
        // junction（删除的是链接本身，不影响目标）或空的真实目录
        std::fs::remove_dir(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// 创建一个指向 `target` 的链接：
/// - Unix: 目录 symlink
/// - Windows: junction（无需管理员权限）
pub fn create_link(target: &Path, link: &Path) -> Result<()> {
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)?;
    }
    #[cfg(windows)]
    {
        junction::create(target, link)?;
    }
    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(target, link)?;
    }
    Ok(())
}
