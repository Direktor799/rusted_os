//! 中断处理子模块
use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::sys_call::sys_call;
use crate::task::{exit_current_and_run_next, get_current_process, schedule_callback};
use core::arch::global_asm;

global_asm!(include_str!("./interrupt.asm"));

const ILLEGAL_INSTRUCTION: usize = 2;
const BREAKPOINT: usize = 3;
const ENVIRONMENT_CALL: usize = 8;
const INSTRUCTION_PAGE_FAULT: usize = 12;
const SUPERVISOR_TIMER_INTERRUPT: usize = (1 << 63) + 5;

/// 初始化中断向量
pub fn init() {
    extern "C" {
        fn __interrupt();
    }
    set_kernel_interrupt();
}

/// 设置内核态中断地址
fn set_kernel_interrupt() {
    unsafe {
        core::arch::asm!("csrw stvec, {}", in(reg) interrupt_kernel as usize);
    };
}

/// 设置用户态中断地址
fn set_user_trap_entry() {
    unsafe {
        core::arch::asm!("csrw stvec, {}", in(reg) TRAMPOLINE as usize);
    };
}

/// 内核态中断处理程序
#[no_mangle]
pub fn interrupt_kernel() -> ! {
    let mut scause: usize;
    let mut stval: usize;
    unsafe {
        core::arch::asm!("csrr {}, scause","csrr {}, stval", out(reg) scause, out(reg) stval);
    }
    panic!(
        "[kernel] Multi-interrupt:\nscause: {:?} stval: {:x}",
        scause, stval
    );
}

/// 用户态中断处理程序
#[no_mangle]
pub fn interrupt_handler() -> ! {
    set_kernel_interrupt();
    let context = get_current_process().inner.borrow_mut().trap_cx();
    let mut scause: usize;
    let mut stval: usize;
    unsafe {
        core::arch::asm!("csrr {}, scause","csrr {}, stval", out(reg) scause, out(reg) stval);
    }
    match scause {
        BREAKPOINT => {
            println!("Breakpoint at 0x{:x}", context.sepc);
            context.sepc += 2;
        }
        SUPERVISOR_TIMER_INTERRUPT => schedule_callback(),
        ENVIRONMENT_CALL => {
            context.sepc += 4;
            let ret_code = sys_call(context.x[17], [context.x[10], context.x[11], context.x[12]]);
            let context = get_current_process().inner.borrow_mut().trap_cx();
            context.x[10] = ret_code as usize;
        }
        ILLEGAL_INSTRUCTION => {
            unsafe {
                let vaddr = crate::memory::frame::address::VirtAddr(context.sepc);
                let token = get_current_process().inner.borrow().token();
                let ppn = crate::memory::frame::page_table::PageTable::from_token(token)
                    .translate(vaddr.vpn())
                    .unwrap();
                let paddr = ppn.addr().0 + vaddr.page_offset();
                println!(
                    "[kernel] IllegalInstruction at 0x{:x}: {}, kernel killed it.",
                    context.sepc,
                    *(paddr as *const u32)
                );
            }
            exit_current_and_run_next(-1);
        }
        INSTRUCTION_PAGE_FAULT => {
            println!(
                "[kernel] InstructionPageFault at 0x{:x}, kernel killed it.",
                context.sepc,
            );
            exit_current_and_run_next(-2);
        }
        _ => {
            panic!(
                "Unresolved interrupt: {:?}\n{:x?}\nstval: {:x}",
                scause, context, stval
            );
        }
    }
    interrupt_return();
}

/// 中断恢复程序
pub fn interrupt_return() -> ! {
    set_user_trap_entry();
    let user_satp = get_current_process().inner.borrow().token();
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
}
