#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use alloc::{string::String, vec};
use console::{get_line, LF};
use core::str;
use user_lib::*;

#[no_mangle]
fn main(args: &[&str]) -> i32 {
    if args.len() == 1 {
        let mut text = String::new();
        loop {
            let s = get_line();
            if s.is_empty() {
                break;
            }
            text.push_str(&s);
        }
        let (lines, words, chars) = count(&text);
        println!("\t{}\t{}\t{}", lines, words, chars);
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
        let (lines, words, chars) = count(str::from_utf8(&buf).unwrap());
        println!("\t{}\t{}\t{}\t{}", lines, words, chars, target);
        close(fd as usize);
    }
    0
}

fn count(text: &str) -> (usize, usize, usize) {
    let lines = text.chars().filter(|ch| *ch == LF).count();
    let words = text.split_ascii_whitespace().count();
    let chars = text.len();
    (lines, words, chars)
}
