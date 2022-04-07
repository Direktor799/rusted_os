#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
mod console;
pub mod batch;
mod config;
mod interrupt;
mod memory;
mod panic;
mod sbi;
mod syscall;
mod timer;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

/// This is where we start.
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    println!("Hello rusted_os!");
    batch::init();
    interrupt::init();
    memory::init();
    unsafe {
        batch::run_next_app();
    }
    panic!("Dummy as fuck");
}
