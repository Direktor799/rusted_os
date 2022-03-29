use super::context::Context;
use super::timer;
use crate::batch::run_next_app;
use crate::syscall::sys_call;
use core::arch::global_asm;
use riscv::register::scause::{Exception, Interrupt, Scause, Trap};

global_asm!(include_str!("./interrupt.asm"));

/// 初始化中断向量
pub fn init() {
    unsafe {
        extern "C" {
            fn __interrupt();
        }
        // stvec::write(__interrupt as usize, stvec::TrapMode::Direct);
        core::arch::asm!(
            "csrw stvec, {}", in(reg) __interrupt as usize
        )
    }
}

/// 中断处理程序
#[no_mangle]
pub fn interrupt_handler(context: &mut Context, scause: Scause, stval: usize) -> &mut Context {
    println!("Interrupted: {:?}", scause.cause());
    match scause.cause() {
        Trap::Exception(Exception::Breakpoint) => breakpoint(context),
        Trap::Interrupt(Interrupt::SupervisorTimer) => supervisor_timer(context),
        Trap::Exception(Exception::UserEnvCall) => {
            context.sepc += 4;
            context.x[10] =
                sys_call(context.x[17], [context.x[10], context.x[11], context.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) => {
            println!("[kernel] StoreFault in application, kernel killed it.");
            run_next_app();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            run_next_app();
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
    context.sepc += 4;
}

fn supervisor_timer(_: &Context) {
    timer::tick();
}
