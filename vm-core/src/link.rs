use std::path::Path;

use crate::Result;

/// 删除一个已存在的链接（symlink / junction），不存在则忽略。
pub fn remove_link(path: &Path) -> Result<()> {
    if path.exists() || is_link(path) {
        #[cfg(windows)]
        {
            // junction 是目录，symlink 文件可能是文件；统一尝试。
            if path.is_dir() {
                std::fs::remove_dir(path)?;
            } else {
                std::fs::remove_file(path)?;
            }
        }
        #[cfg(not(windows))]
        {
            // Unix 上 symlink 用 remove_file 删除（即使是目录链接）。
            std::fs::remove_file(path)?;
        }
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

#[cfg(not(windows))]
fn is_link(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_link(_path: &Path) -> bool {
    true
}
