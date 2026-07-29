use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 整个工具的配置，序列化为 `~/.vm/config.toml`。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// 类目名 -> 类目定义（如 nodejs / java）。
    #[serde(default)]
    pub categories: HashMap<String, Category>,
}

/// 一个类目：一组同类运行时（如所有 JDK 安装）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// 人类可读说明。
    #[serde(default)]
    pub description: String,

    /// 当前激活版本号；未设置时为 None。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<String>,

    /// 版本号 -> 安装信息。
    #[serde(default)]
    pub versions: HashMap<String, VersionEntry>,
}

/// 单个已安装版本的描述。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    /// 安装根目录（用户手动指定）。
    pub path: String,

    /// 显式指定的 bin 目录；为 None 时自动探测 `<path>/bin` 或 `<path>`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bin: Option<String>,
}
