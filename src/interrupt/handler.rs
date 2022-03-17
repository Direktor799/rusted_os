use super::context::Context;
use super::timer;
use core::arch::global_asm;
use riscv::register::{scause, stvec};

global_asm!(include_str!("./interrupt.asm"));

pub fn init() {
    unsafe {
        extern "C" {
            fn __interrupt();
        }
        stvec::write(__interrupt as usize, stvec::TrapMode::Direct);
    }
}

#[no_mangle]
pub fn handle_interrupt(context: &mut Context, scause: scause::Scause, stval: usize) {
    println!("Interrupted: {:?}", scause.cause());
    match scause.cause() {
        scause::Trap::Exception(scause::Exception::Breakpoint) => breakpoint(context),
        scause::Trap::Interrupt(scause::Interrupt::SupervisorTimer) => supervisor_timer(context),
        _ => fault(context, scause, stval),
    }
}

fn breakpoint(context: &mut Context) {
    println!("Breakpoint at 0x{:x}", context.sepc);
    context.sepc += 2;
}

fn supervisor_timer(_: &Context) {
    timer::tick();
}

fn fault(context: &mut Context, scause: scause::Scause, stval: usize) {
    panic!(
        "Unresolved interrupt: {:?}\n{:x?}\nstval: {:x}",
        scause.cause(),
        context,
        stval
    );
}
