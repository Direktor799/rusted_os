#![no_std]
#![no_main]

extern crate alloc;

extern crate user_lib;
use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    if fork() == 0 {
        println!("I am child");
    } else {
        println!("I am father");
    }
    0
}
