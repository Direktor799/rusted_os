#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use alloc::vec::Vec;
use console::get_line;
use core::str;
use user_lib::*;

#[no_mangle]
fn main(args: &[&str]) -> i32 {
    if args.len() != 2 {
        println!("Usage: awk [COLNUM]");
        return 1;
    }
    let col_num = args[1].parse::<usize>().unwrap();
    loop {
        let s = get_line();
        if s.is_empty() {
            break;
        }
        let cols = s.split_ascii_whitespace().collect::<Vec<_>>();
        println!(
            "{}",
            cols.get(col_num).unwrap_or(cols.last().unwrap_or(&""))
        );
    }
    return 0;
}
