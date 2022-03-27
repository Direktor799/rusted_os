#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
mod console;
mod config;
mod interrupt;
mod memory;
mod panic;
mod sbi;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

/// This is where we start.
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    println!("Hello rusted_os!");
    interrupt::init();
    memory::init();
    panic!("Dummy as fuck");
}
