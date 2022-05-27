#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use alloc::vec;
use core::str;
use user_lib::*;

#[no_mangle]
fn main(args: &[&str]) -> i32 {
    let mut targets = vec![];
    if args.len() == 1 {
        targets.push(".");
    }
    for target in &args[1..] {
        targets.push(target);
    }
    for target in targets {
        let fd = open(target, RDONLY);
        let mut stat = Stat::new();
        if fd == -1 {
            println!("cannot access '{}': No such file or directory", target);
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
        match stat.mode as usize {
            REG => {
                println!("{}", target);
            }
            DIR => {
                let mut buf = vec![0u8; stat.size as usize];
                read(fd as usize, &mut buf);
                for i in 2..buf.len() / DIRENT_SZ {
                    let offset = i * DIRENT_SZ;
                    let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const Dirent) };
                    let len = dirent
                        .name
                        .iter()
                        .position(|&v| v == 0)
                        .unwrap_or(dirent.name.len());
                    let name = str::from_utf8(&dirent.name[0..len]).unwrap();
                    print!("{}\t", name);
                }
                if buf.len() / DIRENT_SZ > 2 {
                    println!("");
                }
            }
            _ => panic!("Unknown mode: {}", stat.mode),
        };
        close(fd as usize);
    }
    0
}
