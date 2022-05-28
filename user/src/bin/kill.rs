#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use core::str;
use user_lib::*;

#[no_mangle]
fn main(args: &[&str]) -> i32 {
    for arg in &args[1..] {
        if let Ok(pid) = arg.parse::<isize>() {
            if kill(pid as usize) == -1 {
                println!("({}) - No such process", pid);
            }
        }
    }
    0
}
