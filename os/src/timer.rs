use crate::sbi::set_timer;
use riscv::register::time;

const INTERVAL: usize = 100000; // 时钟中断间隔

/// set next timeout callback
pub fn set_next_timeout() {
    set_timer(time::read() + INTERVAL);
}

/// read the `mtime` register
pub fn get_time() -> usize {
    time::read()
}
