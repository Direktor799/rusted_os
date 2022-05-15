#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(test::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[macro_use]
mod console;
#[macro_use]
mod test;
mod config;
mod drivers;
mod fs;
mod interrupt;
mod loader;
mod memory;
mod panic;
mod sbi;
mod sync;
mod sys_call;
mod task;
use core::arch::global_asm;

use alloc::string::ToString;

use crate::fs::{create_link_by_path, find_inode_by_path, touch_by_path, ROOT_INODE};
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

/// This is where we start.
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    memory::init();
    #[cfg(test)]
    test_main();
    println!("[kernel] Hello rusted_os!");
    interrupt::init();
    loader::init();
    task::init();
    drivers::init();
    fs::format();
    touch_by_path("/a");
    create_link_by_path("/b", "/a");
    create_link_by_path("/c", "/b");

    let inode_a_ = find_inode_by_path("/a");
    let inode_a = inode_a_.as_ref().unwrap();
    
    inode_a.write_at(0, "write_from_a".as_bytes());

    let mut read_buffer = [0u8; 127];
    let mut read_str = alloc::string::String::new();
    let inode_b_ = find_inode_by_path("/b");
    let inode_c_ = find_inode_by_path("/c");
    let inode_b = inode_b_.as_ref().unwrap();
    let inode_c = inode_c_.as_ref().unwrap();
    inode_b.read_at(0, &mut read_buffer);
    read_str = core::str::from_utf8(&read_buffer[..127])
        .unwrap()
        .to_string();
    print!("{}\n",read_str);

    inode_b.write_at(0, "b_write_from_0".as_bytes());
    inode_c.write_at(0, "c_write_from_0".as_bytes());
    inode_b.write_at(3, "a_write_from_3".as_bytes());
    inode_a.read_at(0, &mut read_buffer);
    read_str = core::str::from_utf8(&read_buffer[..127])
        .unwrap()
        .to_string();
    print!("{}\n",read_str);
    // kernel_test_shell();
    panic!("Dummy as fuck");
}

pub fn kernel_test_shell() {
    let mut cur = alloc::string::String::new();
    let mut current_path = alloc::string::String::from("/");
    print!("kernel@rusted_os:{}# ", current_path);
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
            if args[0] == "ls" {
                if args.len() >= 2 {
                    fs::ls_by_path(args[1]);
                } else {
                    fs::ls_by_path(&current_path);
                }
            } else if args[0] == "rm" {
                fs::delete_by_path(args[1]);
            } else if args[0] == "mkdir" {
                fs::mkdir_by_path(args[1])
            } else if args[0] == "touch" {
                fs::touch_by_path(args[1]);
            } else if args[0] == "cd" {
                if fs::check_valid_by_path(args[1]) {
                    current_path = alloc::string::String::from(args[1]);
                } else {
                    println!("{}: No such file or directory", args[1]);
                }
            } else {
                println!("Unknown command");
            }
            cur.clear();
            print!("kernel@rusted_os:{}# ", current_path);
        } else if ch == '`' {
            sbi::shutdown();
        }
    }
}

pub fn get_char() -> char {
    loop {
        let ch = sbi::console_getchar() as u8;
        if ch != 255 {
            return ch as char;
        }
    }
}
