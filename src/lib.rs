pub mod args;
pub mod hash;
pub mod util;
pub use anyhow::Result;

#[macro_export]
macro_rules! async_print {
    ($($arg:tt)*) => (async_std::io::_print(std::format_args!($($arg)*)));
}

#[macro_export]
macro_rules! async_println {
    () => ($crate::async_print!("\n"));
    ($fmt:expr) => ($crate::async_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::async_print!(concat!($fmt, "\n"), $($arg)*));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore]
    fn test_println() {
        async_std::task::block_on(async {
            async_println!("This is a test.").await;
            async_println!().await;
            async_println!("This is a test2.").await;
        });
    }

    #[test]
    fn test_print() {
        async_std::task::block_on(async {
            async_print!("This is a test.").await;
            async_print!("This is a test2.").await;
        });
    }
}
