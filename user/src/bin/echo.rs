#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use user_lib::*;

#[no_mangle]
fn main(args: &[&str]) -> i32 {
    let output = args[1..].join(" ");
    println!("{}", output);
    0
}
