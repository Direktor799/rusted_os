//! 控制台相关操作封装

use crate::sbi::console_putchar;
use core::fmt::{self, Write};

/// 空Stdout结构体，用以实现Write trait
struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

/// 打印格式化字符串
pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

/// 同std::print
#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

/// 同std::println
#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
