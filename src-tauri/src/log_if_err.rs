#[macro_export]
macro_rules! log_if_err {
    ($expr:expr, $($msg:tt)*) => {
        if let Err(e) = $expr {
            log::error!(concat!($($msg)*, ": {:?}"), e)
        }
    };
}
