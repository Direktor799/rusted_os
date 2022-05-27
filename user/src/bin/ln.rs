#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use core::str;
use user_lib::*;

#[no_mangle]
fn main(args: &[&str]) -> i32 {
    if args.len() <= 2 {
        println!("missing file operand");
        return 1;
    }
    let target = args[1];
    let link_path = args[2];
    match symlink(target, link_path) {
        0 => {}
        -1 => println!(
            "failed to create symbolic link '{}': No such file or directory",
            link_path
        ),
        -2 => println!(
            "failed to create symbolic link '{}': File exists",
            link_path
        ),
        _ => panic!(),
    }
    0
}
