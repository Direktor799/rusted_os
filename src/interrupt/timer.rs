use crate::sbi::set_timer;
use riscv::register::{sie, sstatus, time};

const INTERVAL: usize = 100000;
pub static mut TICKS: usize = 0;

pub fn init() {
    unsafe {
        // 开启 STIE，允许时钟中断
        sie::set_stimer();
        // 开启 SIE（不是 sie 寄存器），允许内核态被中断打断
        sstatus::set_sie();
    }
    // 设置下一次时钟中断
    set_next_timeout();
}

fn set_next_timeout() {
    set_timer(time::read() + INTERVAL);
}

pub fn tick() {
    set_next_timeout();
    unsafe {
        TICKS += 1;
        if TICKS % 100 == 0 {
            println!("{} ticks", TICKS);
        }
    }
}
