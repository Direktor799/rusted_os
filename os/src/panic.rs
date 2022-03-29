use crate::sbi::shutdown;
use core::panic::PanicInfo;

/// panic处理
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

#[no_mangle]
extern "C" fn abort() -> ! {
    panic!("abort()")
}
