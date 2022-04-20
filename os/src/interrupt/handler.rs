use super::context::Context;
use crate::config::{TASK_QUEUE_FCFS1_SLICE, TRAMPOLINE, TRAP_CONTEXT};
use crate::syscall::sys_call;
use crate::task::{exit_current_and_run_next, schedule_callback, TASK_MANAGER};
use crate::timer;
use core::arch::global_asm;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Scause, Trap},
    sie, stval, stvec,
};

global_asm!(include_str!("./interrupt.S"));

/// 初始化中断向量
pub fn init() {
    extern "C" {
        fn __interrupt();
    }
    set_kernel_interrupt();
    enable_timer_interrupt();
    timer::set_next_timeout(TASK_QUEUE_FCFS1_SLICE);
}

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

fn set_kernel_interrupt() {
    unsafe { stvec::write(interrupt_kernel as usize, TrapMode::Direct) };
}

fn set_user_trap_entry() {
    unsafe { stvec::write(TRAMPOLINE as usize, TrapMode::Direct) };
}

#[no_mangle]
pub fn interrupt_kernel() -> ! {
    panic!(
        "[kernel] Multi-interrupt: {:?}\nstval: {:x}",
        scause::read().cause(),
        stval::read()
    )
}

/// 中断处理程序
#[no_mangle]
pub fn interrupt_handler() -> ! {
    set_kernel_interrupt();
    let context = unsafe { TASK_MANAGER.get_current_trap_cx() };
    let scause = scause::read();
    let stval = stval::read();
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
    interrupt_return();
}

pub fn interrupt_return() -> ! {
    set_user_trap_entry();
    let user_satp = unsafe { TASK_MANAGER.get_current_token() };
    extern "C" {
        fn __interrupt();
        fn __restore();
    }
    // offset to __restore
    let restore_va = __restore as usize - __interrupt as usize + TRAMPOLINE;
    unsafe {
        core::arch::asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") TRAP_CONTEXT,      // 固定的用户空间context位置
            in("a1") user_satp,
            options(noreturn)
        );
    }
    unreachable!("Unreachable in back_to_user!");
}

fn breakpoint(context: &mut Context) {
    println!("Breakpoint at 0x{:x}", context.sepc);
    context.sepc += 2;
}

fn supervisor_timer(_: &Context) {
    // println!("timer called");
    schedule_callback();
}
