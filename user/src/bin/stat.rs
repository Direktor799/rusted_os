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
        let fd = open(target, RDONLY);
        let mut stat = Stat::new();
        if fd == -1 {
            println!("cannot stat '{}': No such file or directory", target);
            continue;
        }
        match fstat(fd as usize, &mut stat) {
            0 => {}
            -1 => {
                println!("{}: Bad file descriptor", fd);
                continue;
            }
            _ => panic!(),
        }
        close(fd as usize);
        let file_type = match stat.mode as usize {
            CHR => "character special file",
            REG => "regular file",
            DIR => "directory",
            LNK => "symbolic link",
            _ => panic!("Unknown mode: {}", stat.mode),
        };
        println!("File: {}\t\tType: {}", target, file_type);
        println!("Size: {}\t\tInode: {}", stat.size, stat.ino);
    }
    0
}
