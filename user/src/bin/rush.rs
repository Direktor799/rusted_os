#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use alloc::string::String;
use alloc::vec::Vec;
use core::str;
use user_lib::console::get_line;
use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    let mut cwd = String::new();
    let mut ret_code = 0;
    let mut backgroud_pids = Vec::new();
    getcwd(&mut cwd);
    loop {
        for i in (0..backgroud_pids.len()).rev() {
            let ret = waitpid(backgroud_pids[i], &mut ret_code, true);
            if ret > 0 {
                println!("[{}] Done", backgroud_pids.len());
                backgroud_pids.remove(i);
            }
        }
        print!("root@rusted_os:{}# ", cwd);
        let input = get_line();
        let mut args = input.split_ascii_whitespace().collect::<Vec<_>>();
        if !args.is_empty() {
            match args[0] {
                "cd" => cd(&mut cwd, &args),
                "exit" => break,
                _ => {
                    // 判断是否后台运行
                    let background = if args.last().unwrap() == &"&" {
                        args.pop();
                        true
                    } else {
                        false
                    };
                    let pid = fork();
                    if pid == 0 {
                        execute_cmd(args);
                    } else {
                        if background {
                            backgroud_pids.push(pid as usize);
                            println!("[{}] {}", backgroud_pids.len(), pid);
                        } else {
                            waitpid(pid as usize, &mut ret_code, false);
                        }
                    }
                }
            }
        }
    }
    0
}

fn cd(cwd: &mut String, args: &Vec<&str>) {
    let path = match args.len() {
        1 => String::from("/"),
        2 => String::from(args[1]),
        _ => {
            println!("too many arguments");
            return;
        }
    };

    match chdir(&path) {
        0 => {}
        -1 => println!("{}: No such file or directory", args[1]),
        -2 => println!("{}: Not a directory", args[1]),
        _ => panic!(),
    }
    getcwd(cwd);
}

fn execute_cmd(mut args: Vec<&str>) {
    let splited = args
        .rsplitn(2, |ch| *ch == "|")
        .map(|slice| slice.to_vec())
        .collect::<Vec<_>>();
    if splited.len() == 2 {
        args = splited[0].clone();
        let mut pipe_fds = [0; 2];
        pipe(&mut pipe_fds);
        if fork() == 0 {
            dup2(pipe_fds[1], 1);
            close(pipe_fds[0]);
            close(pipe_fds[1]);
            println!("go {:?}", splited[1]);
            execute_cmd(splited[1].clone());
        } else {
            dup2(pipe_fds[0], 0);
            close(pipe_fds[0]);
            close(pipe_fds[1]);
            // wait(&mut ret_code);
        }
    }
    // 输入重定向
    if let Some(input_pos) = args.iter().position(|arg| *arg == "<") {
        if input_pos + 1 >= args.len() {
            println!("syntax error");
            return;
        }
        let fd = open(args[input_pos + 1], RDONLY);
        if fd == -1 {
            println!("'{}': No such file or directory", args[input_pos + 1]);
            return;
        }
        dup2(fd as usize, 0);
        args.drain(input_pos..=input_pos + 1);
    }
    // 输出重定向
    if let Some(output_pos) = args.iter().position(|arg| *arg == ">") {
        if output_pos + 1 >= args.len() {
            println!("syntax error");
            return;
        }
        let fd = open(args[output_pos + 1], WRONLY | CREATE);
        if fd == -1 {
            println!("'{}': No such file or directory", args[output_pos + 1]);
            return;
        }
        dup2(fd as usize, 1);
        args.drain(output_pos..=output_pos + 1);
    }
    exec(&(String::from("/bin/") + args[0]), &args);
    println!("{}: command not found", args[0]);
}
