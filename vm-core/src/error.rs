use thiserror::Error;

/// 工具统一错误类型。
#[derive(Debug, Error)]
pub enum VmError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse config: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("failed to serialize config: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("category '{0}' not found")]
    CategoryNotFound(String),

    #[error("version '{0}' not found in category '{1}'")]
    VersionNotFound(String, String),

    #[error("category '{0}' already exists")]
    CategoryExists(String),

    #[error("version '{0}' already exists in category '{1}'")]
    VersionExists(String, String),

    #[error("no active version set for category '{0}'")]
    NoActiveVersion(String),

    #[error("resolved bin directory does not exist: {0}")]
    BinDirMissing(String),
}

pub type Result<T> = std::result::Result<T, VmError>;
