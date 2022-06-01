#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use alloc::{string::String, vec::Vec};
use console::get_line;
use core::str;
use user_lib::*;

#[no_mangle]
fn main(args: &[&str]) -> i32 {
    let mut exec_args = args[1..].to_vec();
    if exec_args.is_empty() {
        exec_args.push("echo");
    }
    let mut text = String::new();
    loop {
        let s = get_line();
        if s.is_empty() {
            break;
        }
        text.push_str(&s);
    }
    exec_args.append(&mut text.split_ascii_whitespace().collect::<Vec<_>>());
    exec(&(String::from("/bin/") + exec_args[0]), &exec_args);
    return 0;
}
