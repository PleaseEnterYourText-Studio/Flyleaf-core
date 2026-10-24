use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("无法识别的输入：{0}")]
    UnsupportedFormat(String),
    #[error("输入数据无效：{0}")]
    InvalidData(String),
    #[error("I/O 错误：{0}")]
    Io(#[from] std::io::Error),
    #[error("尚未实现：{0}")]
    NotImplemented(&'static str),
    #[error("{0}")]
    Device(String),
    #[error("内部错误：{0}")]
    Internal(String),
}

impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Error::UnsupportedFormat(_) => "unsupported_format",
            Error::InvalidData(_) => "invalid_data",
            Error::Io(_) => "io",
            Error::NotImplemented(_) => "not_implemented",
            Error::Device(_) => "device",
            Error::Internal(_) => "internal",
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
