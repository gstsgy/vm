use std::path::PathBuf;

use crate::{Config, Result};

/// `~/.vm` 配置目录。
pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".vm")
}

/// `~/.vm/config.toml`
pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

/// 加载配置；文件不存在时返回默认（空）配置。
pub fn load() -> Result<Config> {
    let p = config_path();
    if !p.exists() {
        return Ok(Config::default());
    }
    let s = std::fs::read_to_string(&p)?;
    let cfg: Config = toml::from_str(&s)?;
    Ok(cfg)
}

/// 保存配置到 `~/.vm/config.toml`（自动创建目录）。
pub fn save(cfg: &Config) -> Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let s = toml::to_string_pretty(cfg)?;
    std::fs::write(config_path(), s)?;
    Ok(())
}
