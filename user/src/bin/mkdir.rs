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
        match mkdir(target) {
            0 => {}
            -1 => println!(
                "cannot create directory '{}': No such file or directory",
                target
            ),
            -2 => println!("cannot create directory '{}': File exists", target),
            _ => panic!(),
        }
    }
    0
}
