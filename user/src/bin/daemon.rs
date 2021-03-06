#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    if fork() == 0 {
        exec("/bin/rush", &["rush"]);
    } else {
        loop {
            let mut exit_code: i32 = 0;
            let pid = wait(&mut exit_code);
            if pid == -1 {
                r#yield();
                continue;
            }
        }
    }
    0
}
