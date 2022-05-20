#![no_std]
#![no_main]

extern crate alloc;

#[macro_use]
extern crate user_lib;
const RDONLY: u32 = 0;
const WRONLY: u32 = 1 << 0;
const RDWR: u32 = 1 << 1;
const CREATE: u32 = 1 << 9;
const TRUNC: u32 = 1 << 10;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use user_lib::console::get_char;
use user_lib::*;
use core::str;

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
            cur.pop();
            println!("");
            let args = cur.split_whitespace().collect::<alloc::vec::Vec<_>>();
            if !args.is_empty() {
                match args[0] {
                    "cd" => cd(&mut cwd, &args),
                    "mkdir" => app_mkdir(&args),
                    "touch" => app_touch(&args),
                    "read" => read_test(&args),
                    "write" => write_test(&args),
                    _ => println!("{}: command not found1111", args[0]),
                }
            }
            cur.clear();
            print!("root@rusted_os:{}# ", cwd);
        }
    }
    0
}

fn cd(cwd: &mut String, args: &Vec<&str>) {
    let path = match args.len() {
        1 => String::from("/"),
        2 => String::from(args[1]),
        _ => {
            println!("too many arguments");
            return;
        }
    };

    match chdir(&path) {
        0 => {}
        -1 => println!("{}: No such file or directory", args[1]),
        -2 => println!("{}: Not a directory", args[1]),
        _ => panic!(),
    }
    getcwd(cwd);
}

fn app_mkdir(args: &Vec<&str>) {
    if args.len() == 1 {
        println!("missing operand");
        return;
    }
    for target in &args[1..] {
        match mkdir(target) {
            0 => {}
            -1 => println!(
                "cannot create directory '{}': No such file or directory",
                target
            ),
            -2 => println!("cannot create directory '{}': File exists", target),
            _ => panic!(),
        }
    }
}

fn app_touch(args: &Vec<&str>) {
    if args.len() == 1 {
        println!("missing operand");
        return;
    }
    for target in &args[1..] {
        let fd = open(target, CREATE);
        match fd {
            -1 => println!("cannot open file '{}'", target),
            _ => println! {"fd:{}",fd},
        }
    }
}
fn read_test(args: &Vec<&str>) {
    if args.len() == 1 {
        println!("missing operand");
        return;
    }
    let target = args[1];
    let mut buffer = [0u8; 128];
    match read(target.parse::<usize>().unwrap(), &mut buffer) {
        -1 => println!("cannot read file '{}'", target),
        _ => {
            let len = buffer.iter().position(|&v| v == 0).unwrap_or(buffer.len());
            let buf_str = str::from_utf8(&buffer[0..len]).unwrap().to_string();
            println!("read_mess:{}",buf_str);
        }
    }
}
fn write_test(args: &Vec<&str>) {
    if args.len() == 2 {
        println!("missing operand");
        return;
    }
    let target = args[1];
    let buf_str = args[2].clone().as_bytes();

    match write(target.parse::<usize>().unwrap(), buf_str.clone()) {
        -1 => println!("cannot write file '{}'", target.parse::<usize>().unwrap(),),
        _ => {}
    }
}
