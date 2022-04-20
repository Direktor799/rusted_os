#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
mod console;
mod config;
mod interrupt;
mod loader;
mod memory;
mod panic;
mod sbi;
mod syscall;
mod task;
mod timer;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

/// This is where we start.
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    println!("[kernel] Hello rusted_os!");
    interrupt::init();
    memory::init();
    loader::init();
    task::init();
    // let mut cur_time = timer::get_time_ms() / 1000;
    // loop {
    //     let new_time = timer::get_time_ms() / 1000;
    //     if new_time != cur_time {
    //         cur_time = new_time;
    //         println!("{}", new_time);
    //     }
    // }
    task::run();
    panic!("Dummy as fuck");
}
