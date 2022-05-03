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
    fs::init();
    
    let root_inode = unsafe { fs::ROOT_INODE.as_ref().unwrap() };
    // root_inode.clear();
    
    let test2 = root_inode.create("test2", fs::layout::InodeType::Directory).unwrap();
    test2.create("a", fs::layout::InodeType::File);
    test2.create("b", fs::layout::InodeType::File);
    for name in test2.ls() 
    {
        println!("{}",name);
    }
    test2.delete("a");
    for name in test2.ls() 
    {
        println!("{}",name);
    }
    // kernel_test_shell();
    panic!("Dummy as fuck");
}

pub fn kernel_test_shell() {
    let mut cur = alloc::string::String::new();
    loop {
        if cur == "ls" {
            println!("lsing");
            let root_inode = unsafe { fs::ROOT_INODE.as_ref().unwrap() };
            for name in root_inode.ls() {
                println!("{}", name);
            }
            cur.clear();
        }
        let ch = get_char();
        if ch == '`' {
            sbi::shutdown();
        }
        println!("{}", ch);
        cur.push(ch);
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
