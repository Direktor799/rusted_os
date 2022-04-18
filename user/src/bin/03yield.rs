#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    for _ in 0..300 {
        print!("yield");
        user_lib::r#yield();
    }
    0
}
