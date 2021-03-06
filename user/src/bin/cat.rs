#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use alloc::vec;
use console::get_line;
use core::str;
use user_lib::*;

#[no_mangle]
fn main(args: &[&str]) -> i32 {
    if args.len() == 1 {
        loop {
            let s = get_line();
            if s.is_empty() {
                break;
            }
            print!("{}", s);
        }
        return 0;
    }
    for target in &args[1..] {
        // TODO: check file type
        let fd = open(target, RDONLY);
        if fd == -1 {
            println!("{}: No such file or directory", target);
            continue;
        }
        let mut stat = Stat::new();
        match fstat(fd as usize, &mut stat) {
            0 => {}
            -1 => {
                println!("{}: Bad file descriptor", fd);
                continue;
            }
            _ => panic!(),
        }
        let mut buf = vec![0u8; stat.size as usize];
        read(fd as usize, &mut buf);
        print!("{}", str::from_utf8(&buf[0..stat.size as usize]).unwrap());
        close(fd as usize);
    }
    0
}
