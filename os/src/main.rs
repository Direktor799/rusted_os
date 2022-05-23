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
mod memory;
mod panic;
mod sbi;
mod sys_call;
mod task;
mod tools;
use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));

/// This is where we start.
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    memory::init();
    interrupt::init();
    drivers::init();
    fs::init();
    task::init();
    #[cfg(test)]
    test_main();
    println!("[kernel] Hello rusted_os!");

    unsafe {
        fs::rfs::ROOT_INODE.ls();
    }

    task::run();
    panic!("Dummy as fuck");
}
