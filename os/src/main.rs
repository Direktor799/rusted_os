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
mod interrupt;
mod loader;
mod memory;
mod panic;
mod sbi;
mod syscall;
mod task;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

/// This is where we start.
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    #[cfg(test)]
    test_main();
    println!("[kernel] Hello rusted_os!");
    memory::init();
    interrupt::init();
    loader::init();
    task::init();
    system_test!(_test_a_plus_b);
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

fn _test_a_plus_b() {
    assert_eq!(1 + 1, 2);
    println!("Correct!");
}
