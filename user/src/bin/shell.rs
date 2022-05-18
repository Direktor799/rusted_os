#![no_std]
#![no_main]

extern crate alloc;

#[macro_use]
extern crate user_lib;

use alloc::string::String;
use user_lib::console::get_char;
use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    let mut cur = alloc::string::String::new();
    let mut cwd = String::new();
    getcwd(&mut cwd);
    print!("root@rusted_os:{}# ", cwd);
    loop {
        let ch = get_char();
        print!("{}", ch);
        if ch == '\x7f' {
            if !cur.is_empty() {
                print!("\x08 \x08");
                cur.pop();
            }
            continue;
        }
        cur.push(ch);
        if ch == '\r' {
            println!("");
            let args = cur.split_whitespace().collect::<alloc::vec::Vec<_>>();
            if args[0] == "cd" {
                if chdir(args[1]) != 0 {
                    println!("{}: No such file or directory", args[1]);
                }
                getcwd(&mut cwd);
            } else {
                println!("{}: command not found", cur);
            }
            cur.clear();
            print!("root@rusted_os:{}# ", cwd);
        }
    }
    0
}
