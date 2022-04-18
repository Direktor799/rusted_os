use crate::task::{exit_current_and_run_next, schedule_callback};

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exit with code {}", exit_code);
    exit_current_and_run_next(exit_code);
    // panic!("Unreachable in sys_exit!");
    println!("waiting");
    loop {}
}

pub fn sys_yield() -> isize {
    schedule_callback();
    0
}
