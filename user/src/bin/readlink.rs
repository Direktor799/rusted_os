#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use alloc::string::String;
use core::str;
use user_lib::*;

#[no_mangle]
fn main(args: &[&str]) -> i32 {
    if args.len() == 1 {
        println!("missing operand");
        return 1;
    }
    for target in &args[1..] {
        let mut content = String::new();
        if readlink(target, &mut content) != 0 {
            println!("{}", content);
        }
    }
    0
}
