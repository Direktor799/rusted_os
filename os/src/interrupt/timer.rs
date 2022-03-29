use crate::sbi::set_timer;
use riscv::register::{sie, sstatus, time};

const INTERVAL: usize = 100000; // 时钟中断间隔
pub static mut TICKS: usize = 0; // 时钟中断次数

pub fn init() {
    unsafe {
        // 开启 STIE，允许时钟中断
        sie::set_stimer();
        // 开启 SIE，允许内核态被中断打断
        sstatus::set_sie();
    }
    // 设置第一次时钟中断
    set_next_timeout();
}

fn set_next_timeout() {
    set_timer(time::read() + INTERVAL);
}

/// 设置下一次时钟中断
pub fn tick() {
    set_next_timeout();
    unsafe {
        TICKS += 1;
        if TICKS % 100 == 0 {
            // println!("{} ticks", TICKS);
        }
    }
}
