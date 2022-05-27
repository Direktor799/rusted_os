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
    getcwd(&mut cwd);
    loop {
        print!("root@rusted_os:{}# ", cwd);
        let input = get_line();
        let args = input.split_whitespace().collect::<Vec<_>>();
        if !args.is_empty() {
            match args[0] {
                "cd" => cd(&mut cwd, &args),
                "mkdir" => run("/bin/mkdir", &args, &mut ret_code),
                "cat" => run("/bin/cat", &args, &mut ret_code),
                "ln" => run("/bin/ln", &args, &mut ret_code),
                "readlink" => run("/bin/readlink", &args, &mut ret_code),
                "rm" => run("/bin/rm", &args, &mut ret_code),
                "rmdir" => run("/bin/rmdir", &args, &mut ret_code),
                "stat" => run("/bin/stat", &args, &mut ret_code),
                "ls" => run("/bin/ls", &args, &mut ret_code),
                "write" => write_test(&args),
                "exit" => break,
                _ => println!("{}: command not found", args[0]),
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

fn run(path: &str, args: &[&str], ret_code: &mut i32) {
    let pid = fork();
    if pid == 0 {
        exec(path, args);
    } else {
        waitpid(pid as usize, ret_code);
    }
}

fn write_test(args: &Vec<&str>) {
    if args.len() <= 2 {
        println!("missing operand");
        return;
    }
    let target = args[1];
    let buf_str = args[2].as_bytes();
    let fd = open(target, WRONLY | CREATE);
    if fd == -1 {
        println!("{}: No such file or directory", target);
        return;
    }
    let len = write(fd as usize, buf_str);
    match len {
        -1 => println!("{}: Bad file descriptor", fd),
        _ => println!("ok"),
    }
    close(fd as usize);
}
