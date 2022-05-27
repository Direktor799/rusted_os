#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use core::str;
use user_lib::*;

#[no_mangle]
fn main(args: &[&str]) -> i32 {
    let mut buffer = [0u8; 128];
    if args.len() == 1 {
        println!("cat for stdin not supported");
        return 1;
    }
    for target in &args[1..] {
        // TODO: check file type
        let fd = open(target, RDONLY);
        if fd == -1 {
            println!("{}: No such file or directory", target);
            continue;
        }
        loop {
            let len = read(fd as usize, &mut buffer);
            match len {
                0 => break,
                -1 => {
                    println!("{}: Bad file descriptor", fd);
                    break;
                }
                _ => print!("{}", str::from_utf8(&buffer[0..len as usize]).unwrap()),
            }
        }
        close(fd as usize);
    }
    0
}
