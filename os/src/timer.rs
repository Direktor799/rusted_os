use crate::sbi::set_timer;
use riscv::register::time;

/// set next timeout callback
pub fn set_next_timeout(interval: usize) {
    set_timer(time::read() + interval);
}

/// read the `mtime` register
pub fn get_time() -> usize {
    time::read()
}
