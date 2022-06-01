#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use alloc::{string::String, vec, vec::Vec};
use console::get_line;
use core::str;
use user_lib::*;

#[no_mangle]
fn main(args: &[&str]) -> i32 {
    if args.len() == 1 {
        println!("Usage: grep PATTERNS [FILE]...");
        return 1;
    }
    let pattern = args[1];
    if args.len() == 2 {
        let mut text = String::new();
        loop {
            let s = get_line();
            if s.is_empty() {
                break;
            }
            text.push_str(&s);
        }
        let res = find(&text, pattern);
        for line in res {
            println!("{}", line);
        }
        return 0;
    }
    for target in &args[2..] {
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
        let res = find(str::from_utf8(&buf).unwrap(), pattern);
        for line in res {
            println!("{}:{}", target, line);
        }
        close(fd as usize);
    }
    0
}

fn find<'a>(text: &'a str, pattern: &str) -> Vec<&'a str> {
    let lines = text.split("\n").collect::<Vec<_>>();
    let mut res = vec![];
    for line in lines {
        if line.find(pattern).is_some() {
            res.push(line);
        }
    }
    res
}
