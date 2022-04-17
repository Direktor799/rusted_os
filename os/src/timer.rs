use crate::sbi::set_timer;
use riscv::register::time;

const INTERVAL: usize = 100000; // 时钟中断间隔

pub fn set_next_timeout() {
    set_timer(time::read() + INTERVAL);
}
