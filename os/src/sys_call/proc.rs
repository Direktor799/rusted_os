//! 进程相关系统调用子模块

use alloc::rc::Rc;
use alloc::vec;

use crate::fs::rfs::{find_inode, get_full_path};
use crate::interrupt::timer::get_time_ms;
use crate::memory::frame::user_buffer::{get_user_string, get_user_value, put_user_value};
use crate::task::{
    add_new_task, exit_current_and_run_next, get_current_process, suspend_current_and_run_next,
    TaskStatus, TASK_MANAGER,
};

pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_gettime() -> isize {
    get_time_ms() as isize
}

pub fn sys_getpid() -> isize {
    let proc = get_current_process();
    proc.pid.0 as isize
}

pub fn sys_fork() -> isize {
    let proc = get_current_process();
    let new_proc = proc.fork();
    let trap_cx = new_proc.inner.borrow().trap_cx();
    trap_cx.x[10] = 0;
    add_new_task(new_proc.clone());
    new_proc.pid.0 as isize
}

pub fn sys_exec(path: *const u8, mut arg_ptrs_ptr: *const *const u8) -> isize {
    let proc = get_current_process();
    let proc_inner = proc.inner.borrow();
    let path = get_user_string(proc_inner.token(), path);
    let path = get_full_path(&proc_inner.cwd, &path);
    let mut args = vec![];
    loop {
        let mut arg_ptr = 0usize;
        get_user_value(proc_inner.token(), arg_ptrs_ptr as *const _, &mut arg_ptr);
        if arg_ptr == 0 {
            break;
        }
        args.push(get_user_string(proc_inner.token(), arg_ptr as *const u8));
        unsafe {
            arg_ptrs_ptr = arg_ptrs_ptr.add(1);
        }
    }
    drop(proc_inner);

    if let Some(app_inode) = find_inode(&path) {
        let size = app_inode.get_file_size() as usize;
        let mut app_data = vec![0u8; size];
        app_inode.read_at(0, &mut app_data);
        proc.exec(&app_data, &args);
        // return argc because cx.x[10] will be covered with it later
        args.len() as isize
    } else {
        -1
    }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut u8) -> isize {
    let process = get_current_process();
    let mut inner = process.inner.borrow_mut();

    if !inner
        .children
        .iter()
        .any(|child| pid == -1 || pid as usize == child.pid.0)
    {
        // child not found
        return -1;
    }

    let pair = inner.children.iter().enumerate().find(|(_, child)| {
        child.inner.borrow().task_status == TaskStatus::Exited
            && (pid == -1 || pid as usize == child.pid.0)
    });

    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        let child_pid = child.pid.0;
        put_user_value(inner.token(), child.inner.borrow().exit_code, exit_code_ptr);
        child_pid as isize
    } else {
        // child running
        -2
    }
}

pub fn sys_kill(pid: usize) -> isize {
    if get_current_process().pid.0 == pid {
        exit_current_and_run_next(0);
        return 0;
    }
    unsafe {
        if let Some(proc) = TASK_MANAGER.tasks().find(|task| task.pid.0 == pid) {
            proc.inner.borrow_mut().task_status = TaskStatus::Exited;
            return 0;
        }
    }
    -1
}
