use std::fmt::Debug;

pub trait LogErr: Sized {
    /// 记录错误
    fn log_err(self, msg: &str) -> Self;
    /// 记录警告
    fn log_warn(self, msg: &str) -> Self;
}

impl<T, E: Debug> LogErr for Result<T, E> {
    fn log_err(self, msg: &str) -> Self {
        if let Err(e) = &self {
            tracing::error!(err = ?e, "{msg}");
        }
        self
    }
    fn log_warn(self, msg: &str) -> Self {
        if let Err(e) = &self {
            tracing::warn!(err = ?e, "{msg}");
        }
        self
    }
}

impl<T> LogErr for Option<T> {
    fn log_err(self, msg: &str) -> Self {
        if self.is_none() {
            tracing::error!("{msg}");
        }
        self
    }
    fn log_warn(self, msg: &str) -> Self {
        if self.is_none() {
            tracing::warn!("{msg}");
        }
        self
    }
}
