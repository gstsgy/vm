//! vm-core: 平台无关的版本管理核心逻辑。
//! 同时被 `vm-cli`（命令行）与 `vm-gui`（Tauri 后端）复用。

mod config;
mod env;
mod error;
mod link;
mod model;
mod switch;

pub use config::{config_dir, config_path, load, save};
pub use env::{add_to_user_path, remove_from_user_path};
pub use error::{Result, VmError};
pub use link::{create_link, remove_link};
pub use model::{Category, Config, VersionEntry};
pub use switch::{
    add_category, add_version, cleanup_category_dir, current_version, init_category_dir,
    link_active_bin, list_categories, list_versions, remove_category, remove_version,
    resolve_bin_dir, use_version,
};
