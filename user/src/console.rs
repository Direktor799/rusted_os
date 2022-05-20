use super::{read, write};
use alloc::string::String;
use core::fmt::{self, Write};

struct Stdout;

const STDIN: usize = 0;
const STDOUT: usize = 1;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write(STDOUT, s.as_bytes());
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

pub fn get_char() -> char {
    let mut c = [0u8; 1];
    while read(STDIN, &mut c) == 0 {}
    c[0] as char
}

const BS: char = '\x08';
const LF: char = '\x0a';
const CR: char = '\x0d';
const DEL: char = '\x7f';

pub fn get_line() -> String {
    let mut input = String::new();
    loop {
        let ch = get_char();
        if ch == DEL {
            if !input.is_empty() {
                input.pop();
                print!("{BS} {BS}");
            }
            continue;
        }
        input.push(ch);
        print!("{}", ch);
        if ch == CR {
            print!("{LF}");
            break input;
        }
    }
}
