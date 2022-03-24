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
    unsafe {
        for _ in 0..2 {
            let frame_0 = match memory::frame::FRAME_ALLOCATOR.borrow_mut().alloc() {
                Option::Some(frame_tracker) => frame_tracker,
                Option::None => panic!("None"),
            };
            let frame_1 = match memory::frame::FRAME_ALLOCATOR.borrow_mut().alloc() {
                Option::Some(frame_tracker) => frame_tracker,
                Option::None => panic!("None"),
            };
            println!("{:?} and {:?}", frame_0.address(), frame_1.address());
        }
    }
    panic!("Dummy as fuck");
}
