//! 计时器操作子模块

use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
use crate::task::schd::get_default_time_slice;

/// 读取time寄存器
pub fn get_time() -> usize {
    let mut time: usize;
    unsafe {
        core::arch::asm!("csrr {}, time", out(reg) time);
    }
    time
}

/// 获取系统时钟(ms)
pub fn get_time_ms() -> usize {
    get_time() / (CLOCK_FREQ / 1000)
}

/// 开启时钟中断
pub fn enable_timer_interrupt() {
    unsafe {
        // set STIE bit
        core::arch::asm!("csrw sie, {}", in(reg) 1 << 5);
    }
}

/// 设置下一个时钟间隔
pub fn set_next_timeout(interval: usize) {
    set_timer(get_time() + interval);
}

/// 时钟初始化
pub fn init() {
    enable_timer_interrupt();
    set_next_timeout(get_default_time_slice());
}
