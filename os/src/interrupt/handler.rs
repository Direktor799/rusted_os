use super::context::Context;
use crate::syscall::sys_call;
use crate::task::{exit_current_and_run_next, schedule_callback};
use crate::timer;
use core::arch::global_asm;
use riscv::register::{
    mtvec::TrapMode,
    scause::{Exception, Interrupt, Scause, Trap},
    sie, stval, stvec,
};

global_asm!(include_str!("./interrupt.S"));

/// 初始化中断向量
pub fn init() {
    extern "C" {
        fn __interrupt();
    }
    // stvec::write(__interrupt as usize, stvec::TrapMode::Direct);
    unsafe {
        stvec::write(__interrupt as usize, TrapMode::Direct);
        enable_timer_interrupt();
        timer::set_next_timeout(1000);
    }
}

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

/// 中断处理程序
#[no_mangle]
pub fn interrupt_handler(context: &mut Context, scause: Scause, stval: usize) -> &mut Context {
    match scause.cause() {
        Trap::Exception(Exception::Breakpoint) => breakpoint(context),
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            supervisor_timer(context);
        }
        Trap::Exception(Exception::UserEnvCall) => {
            context.sepc += 4;
            context.x[10] =
                sys_call(context.x[17], [context.x[10], context.x[11], context.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) => {
            println!("[kernel] StoreFault in application, kernel killed it.");
            exit_current_and_run_next(-1);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!(
                "[kernel] IllegalInstruction at {:x}: {:x}, kernel killed it.",
                context.sepc,
                unsafe { *(context.sepc as *const usize) }
            );
            exit_current_and_run_next(-1);
        }
        _ => {
            panic!(
                "Unresolved interrupt: {:?}\n{:x?}\nstval: {:x}",
                scause.cause(),
                context,
                stval
            );
        }
    }
    context
}

fn breakpoint(context: &mut Context) {
    println!("Breakpoint at 0x{:x}", context.sepc);
    context.sepc += 2;
}

fn supervisor_timer(_: &Context) {
    // println!("timer called");
    schedule_callback();
}
