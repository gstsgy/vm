use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    link, Category, Config, Result, VmError, VersionEntry, config_dir,
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
            bin,
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
        Some(bin) => PathBuf::from(bin),
        None => {
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
}

/// 重建 `~/.vm/<category>/bin` 链接，指向当前激活版本的 bin 目录。
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
    let bin_dir = resolve_bin_dir(entry)?;
    let link = config_dir().join(category).join("bin");
    link::remove_link(&link)?;
    link::create_link(&bin_dir, &link)?;
    Ok(())
}
