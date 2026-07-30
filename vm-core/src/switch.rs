use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    env, link, Category, Config, Result, VmError, VersionEntry, config_dir,
};

/// 新增类目（已存在则报错）。
pub fn add_category(cfg: &mut Config, name: &str, desc: &str) -> Result<()> {
    if cfg.categories.contains_key(name) {
        return Err(VmError::CategoryExists(name.to_string()));
    }
    cfg.categories.insert(
        name.to_string(),
        Category {
            description: desc.to_string(),
            active: None,
            versions: HashMap::new(),
        },
    );
    Ok(())
}

/// 编辑类目说明。
pub fn edit_category(cfg: &mut Config, name: &str, desc: &str) -> Result<()> {
    let cat = cfg
        .categories
        .get_mut(name)
        .ok_or_else(|| VmError::CategoryNotFound(name.to_string()))?;
    cat.description = desc.to_string();
    Ok(())
}

/// 将 bin 目录参数归一化：空串或纯空白视为「自动探测」（None）。
fn normalize_bin(bin: Option<String>) -> Option<String> {
    match bin {
        Some(s) if !s.trim().is_empty() => Some(s),
        _ => None,
    }
}

/// 编辑版本的安装路径与 bin 目录。
/// 若编辑的是当前激活版本，调用方需随后执行 `link_active_bin` 重建链接。
pub fn edit_version(
    cfg: &mut Config,
    category: &str,
    version: &str,
    path: &str,
    bin: Option<String>,
) -> Result<()> {
    let cat = cfg
        .categories
        .get_mut(category)
        .ok_or_else(|| VmError::CategoryNotFound(category.to_string()))?;
    let entry = cat
        .versions
        .get_mut(version)
        .ok_or_else(|| VmError::VersionNotFound(version.to_string(), category.to_string()))?;
    entry.path = path.to_string();
    entry.bin = normalize_bin(bin);
    Ok(())
}

/// 删除类目及其全部版本记录（不会删除用户真实的安装目录）。
pub fn remove_category(cfg: &mut Config, name: &str) -> Result<()> {
    cfg.categories
        .remove(name)
        .ok_or_else(|| VmError::CategoryNotFound(name.to_string()))?;
    Ok(())
}

/// 为类目添加版本（已存在则报错）。
pub fn add_version(
    cfg: &mut Config,
    category: &str,
    version: &str,
    path: &str,
    bin: Option<String>,
) -> Result<()> {
    let cat = cfg
        .categories
        .get_mut(category)
        .ok_or_else(|| VmError::CategoryNotFound(category.to_string()))?;
    if cat.versions.contains_key(version) {
        return Err(VmError::VersionExists(
            version.to_string(),
            category.to_string(),
        ));
    }
    cat.versions.insert(
        version.to_string(),
        VersionEntry {
            path: path.to_string(),
            bin: normalize_bin(bin),
        },
    );
    Ok(())
}

/// 删除某类目下的版本。
pub fn remove_version(cfg: &mut Config, category: &str, version: &str) -> Result<()> {
    let cat = cfg
        .categories
        .get_mut(category)
        .ok_or_else(|| VmError::CategoryNotFound(category.to_string()))?;
    cat.versions
        .remove(version)
        .ok_or_else(|| VmError::VersionNotFound(version.to_string(), category.to_string()))?;
    if cat.active.as_deref() == Some(version) {
        cat.active = None;
    }
    Ok(())
}

/// 切换当前激活版本（仅改配置，链接重建见 `link_active_bin`）。
pub fn use_version(cfg: &mut Config, category: &str, version: &str) -> Result<()> {
    let cat = cfg
        .categories
        .get_mut(category)
        .ok_or_else(|| VmError::CategoryNotFound(category.to_string()))?;
    if !cat.versions.contains_key(version) {
        return Err(VmError::VersionNotFound(
            version.to_string(),
            category.to_string(),
        ));
    }
    cat.active = Some(version.to_string());
    Ok(())
}

/// 查询当前激活版本号。
pub fn current_version(cfg: &Config, category: &str) -> Result<Option<String>> {
    let cat = cfg
        .categories
        .get(category)
        .ok_or_else(|| VmError::CategoryNotFound(category.to_string()))?;
    Ok(cat.active.clone())
}

pub fn list_categories(cfg: &Config) -> Vec<&str> {
    let mut names: Vec<&str> = cfg.categories.keys().map(|s| s.as_str()).collect();
    names.sort_unstable();
    names
}

pub fn list_versions(cfg: &Config, category: &str) -> Result<Vec<String>> {
    let cat = cfg
        .categories
        .get(category)
        .ok_or_else(|| VmError::CategoryNotFound(category.to_string()))?;
    let mut names: Vec<String> = cat.versions.keys().cloned().collect();
    names.sort_unstable();
    Ok(names)
}

/// 根据 VersionEntry 解析出真正包含可执行文件的 bin 目录。
pub fn resolve_bin_dir(entry: &VersionEntry) -> Result<PathBuf> {
    let root = PathBuf::from(&entry.path);
    let candidate = match &entry.bin {
        Some(bin) if !bin.trim().is_empty() => PathBuf::from(bin),
        _ => {
            let with_bin = root.join("bin");
            if with_bin.is_dir() {
                with_bin
            } else {
                root
            }
        }
    };
    if !candidate.exists() {
        return Err(VmError::BinDirMissing(candidate.display().to_string()));
    }
    Ok(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_cfg() -> Config {
        Config::default()
    }

    #[test]
    fn category_lifecycle() {
        let mut cfg = empty_cfg();
        add_category(&mut cfg, "nodejs", "Node").unwrap();
        assert!(add_category(&mut cfg, "nodejs", "dup").is_err());
        assert_eq!(list_categories(&cfg), vec!["nodejs"]);
        remove_category(&mut cfg, "nodejs").unwrap();
        assert!(list_categories(&cfg).is_empty());
    }

    #[test]
    fn version_and_use() {
        let mut cfg = empty_cfg();
        add_category(&mut cfg, "java", "JDK").unwrap();
        add_version(&mut cfg, "java", "21", "/opt/jdk21", None).unwrap();
        add_version(&mut cfg, "java", "17", "/opt/jdk17", None).unwrap();
        assert_eq!(list_versions(&cfg, "java").unwrap(), vec!["17", "21"]);
        // 不能切到不存在的版本
        assert!(use_version(&mut cfg, "java", "99").is_err());
        use_version(&mut cfg, "java", "21").unwrap();
        assert_eq!(current_version(&cfg, "java").unwrap(), Some("21".into()));
        // 删除当前版本应清空 active
        remove_version(&mut cfg, "java", "21").unwrap();
        assert_eq!(current_version(&cfg, "java").unwrap(), None);
    }

    #[test]
    fn bin_dir_resolution() {
        let tmp = std::env::temp_dir().join("vm_test_bin");
        let _ = std::fs::create_dir_all(tmp.join("bin"));
        let entry = VersionEntry {
            path: tmp.to_str().unwrap().to_string(),
            bin: None,
        };
        assert_eq!(resolve_bin_dir(&entry).unwrap(), tmp.join("bin"));

        let explicit = VersionEntry {
            path: tmp.to_str().unwrap().to_string(),
            bin: Some(tmp.to_str().unwrap().to_string()),
        };
        assert_eq!(resolve_bin_dir(&explicit).unwrap(), tmp);
    }

    #[test]
    fn bin_none_normalization() {
        let mut cfg = Config::default();
        add_category(&mut cfg, "nodejs", "Node").unwrap();
        // 空串应被归一化为自动探测（None）
        add_version(&mut cfg, "nodejs", "20", "/opt/node20", Some("".into())).unwrap();
        assert_eq!(cfg.categories["nodejs"].versions["20"].bin, None);
        // 纯空白同样归一化
        edit_version(&mut cfg, "nodejs", "20", "/opt/node20", Some("   ".into())).unwrap();
        assert_eq!(cfg.categories["nodejs"].versions["20"].bin, None);
        // 非空串保留
        edit_version(&mut cfg, "nodejs", "20", "/opt/node20", Some("/opt/node20/bin".into())).unwrap();
        assert_eq!(
            cfg.categories["nodejs"].versions["20"].bin,
            Some("/opt/node20/bin".into())
        );
    }

    #[test]
    fn empty_bin_resolves_like_none() {
        // resolve_bin_dir 对空串 bin 应回退到自动探测，而非当成 cwd
        let tmp = std::env::temp_dir().join("vm_test_bin_empty");
        let _ = std::fs::create_dir_all(tmp.join("bin"));
        let entry = VersionEntry {
            path: tmp.to_str().unwrap().to_string(),
            bin: Some("".into()),
        };
        assert_eq!(resolve_bin_dir(&entry).unwrap(), tmp.join("bin"));
    }
}

/// 初始化类目的磁盘环境：创建 `~/.vm/<类目>` 目录，
/// 并把 `~/.vm/<类目>` 与 `~/.vm/<类目>/bin` 两条都加入用户 PATH。
/// 这样无论可执行文件在版本根目录还是 bin/ 子目录，切换后都能命中。
/// 新增类目后调用；重复调用安全（幂等）。
pub fn init_category_dir(name: &str) -> Result<PathBuf> {
    let dir = config_dir().join(name);
    if std::fs::symlink_metadata(&dir).is_err() {
        std::fs::create_dir_all(&dir)?;
    }
    env::add_to_user_path(&dir)?;
    env::add_to_user_path(&dir.join("bin"))?;
    Ok(dir)
}

/// 清理类目的磁盘环境：删除 `~/.vm/<类目>` 链接/目录，并从用户 PATH 移除两条记录。
/// 删除类目后调用；不会碰用户真实的安装目录（junction 只删链接本身）。
pub fn cleanup_category_dir(name: &str) -> Result<()> {
    let dir = config_dir().join(name);
    if link::remove_link(&dir).is_err() {
        // 兼容旧布局：真实目录里残留 `bin` 链接导致目录非空
        let _ = link::remove_link(&dir.join("bin"));
        link::remove_link(&dir)?;
    }
    env::remove_from_user_path(&dir.join("bin"))?;
    env::remove_from_user_path(&dir)?;
    Ok(())
}

/// 重建 `~/.vm/<category>` 链接（junction/symlink），指向当前激活版本的**安装根目录**。
/// PATH 中已有 `~/.vm/<类目>` 与 `~/.vm/<类目>/bin` 两条，
/// 因此可执行文件在根目录或 bin/ 子目录都可直接使用；
/// `~/.vm/<类目>` 还可以直接用作 JAVA_HOME 之类的 HOME 变量。
/// 这是 `vm use` 切换版本后让 PATH 生效的核心步骤。
pub fn link_active_bin(cfg: &Config, category: &str) -> Result<()> {
    let cat = cfg
        .categories
        .get(category)
        .ok_or_else(|| VmError::CategoryNotFound(category.to_string()))?;
    let active = cat
        .active
        .as_ref()
        .ok_or_else(|| VmError::NoActiveVersion(category.to_string()))?;
    let entry = cat
        .versions
        .get(active)
        .ok_or_else(|| VmError::VersionNotFound(active.clone(), category.to_string()))?;
    // 链接目标 = 安装根目录（bin/ 通过 PATH 第二条记录覆盖）
    let target = PathBuf::from(&entry.path);
    if !target.exists() {
        return Err(VmError::BinDirMissing(target.display().to_string()));
    }
    let link = config_dir().join(category);
    if link::remove_link(&link).is_err() {
        // 兼容旧布局（~/.vm/<类目>/bin 链接残留导致真实目录非空）：先删内层链接再删目录
        let _ = link::remove_link(&link.join("bin"));
        link::remove_link(&link)?;
    }
    link::create_link(&target, &link)?;
    // 确保 PATH 中有两条记录（老版本创建的类目没加过）
    env::add_to_user_path(&link)?;
    env::add_to_user_path(&link.join("bin"))?;
    Ok(())
}
