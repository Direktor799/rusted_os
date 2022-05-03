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
    kernel_test_shell();
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
