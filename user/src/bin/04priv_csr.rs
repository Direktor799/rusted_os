#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use core::arch::asm;

#[no_mangle]
fn main() -> i32 {
    println!("Try to access privileged CSR in U Mode");
    println!("Kernel should kill this application!");
    unsafe {
        asm!(
            "csrw sstatus, {}", in(reg) 0,
        )
    }
    0
}