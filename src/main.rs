#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
mod console;
mod interrupt;
mod memory;
mod panic;
mod sbi;

extern crate alloc;
use alloc::boxed::Box;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

/// This is where we start.
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    println!("Hello rusted_os!");
    interrupt::init();
    memory::init();
    {
        let _x = Box::new(1);
        {
            let _x = Box::new(1);
            let _y = Box::new(1);
        }
    }
    // unsafe {
    //     core::arch::asm!("ebreak");
    // }
    panic!("Dummy as fuck");
}
