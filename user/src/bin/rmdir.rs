#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use core::str;
use user_lib::*;

#[no_mangle]
fn main(args: &[&str]) -> i32 {
    if args.len() == 1 {
        println!("missing operand");
        return 1;
    }
    for target in &args[1..] {
        match unlink(target, AT_REMOVEDIR) {
            0 => {}
            -1 => println!("failed to remove '{}': No such file or directory", target),
            -2 => println!("failed to remove '{}': Not a directory", target),
            -3 => println!("failed to remove '{}': Directory not empty", target),
            _ => panic!(),
        }
    }
    0
}
