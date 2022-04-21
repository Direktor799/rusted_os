//! 全局panic处理

use crate::sbi::shutdown;
use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "\x1b[1;31mPanicked at {}:{} '{}'\x1b[0m",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("\x1b[1;31mPanicked: '{}'\x1b[0m", info.message().unwrap());
    }
    shutdown()
}
