#![no_std]
#![no_main]

extern crate alloc;

extern crate user_lib;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::str;
use user_lib::console::get_line;
use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    let mut cwd = String::new();
    getcwd(&mut cwd);
    loop {
        print!("root@rusted_os:{}# ", cwd);
        let input = get_line();
        let args = input.split_whitespace().collect::<alloc::vec::Vec<_>>();
        if !args.is_empty() {
            match args[0] {
                "cd" => cd(&mut cwd, &args),
                "mkdir" => app_mkdir(&args),
                "cat" => app_cat(&args),
                "cats" => read_from_symlink(&args),
                "write" => write_test(&args),
                "ln" => app_ln(&args),
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

fn app_mkdir(args: &Vec<&str>) {
    if args.len() == 1 {
        println!("missing operand");
        return;
    }
    for target in &args[1..] {
        match mkdir(target) {
            0 => {}
            -1 => println!(
                "cannot create directory '{}': No such file or directory",
                target
            ),
            -2 => println!("cannot create directory '{}': File exists", target),
            _ => panic!(),
        }
    }
}

fn app_cat(args: &Vec<&str>) {
    let mut buffer = [0u8; 128];
    if args.len() == 1 {
        // TODO: maybe support this after sig & dev ?
        println!("cat for stdin not supported");
        return;
    }
    for target in &args[1..] {
        // TODO: check file type
        let fd = open(target, RDONLY);
        if fd == -1 {
            println!("{}: No such file or directory", target);
            continue;
        }
        loop {
            let len = read(fd as usize, &mut buffer);
            match len {
                -1 => {
                    println!("{}: Bad file descriptor", fd);
                    break;
                }
                0 => break,
                _ => print!("{}", str::from_utf8(&buffer[0..len as usize]).unwrap()),
            }
        }
        close(fd as usize);
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
        -1 => println!("{}: Bad file descriptor", target),
        _ => println!("ok"),
    }
    close(fd as usize);
}

fn app_ln(args: &Vec<&str>) {
    if args.len() <= 2 {
        println!("missing file operand");
        return;
    }
    let target = args[1];
    let link_path = args[2];
    match symlink(target, link_path) {
        0 => {}
        -1 => println!(
            "failed to create symbolic link '{}': No such file or directory",
            link_path
        ),
        -2 => println!(
            "failed to create symbolic link '{}': File exists",
            link_path
        ),
        _ => panic!(),
    }
}

fn read_from_symlink(args: &Vec<&str>) {
    // TODO: check file type
    let target = args[1];
    let fd = open(target, RDONLY);
    if fd == -1 {
        println!("{}: No such file or directory", target);
        return;
    }
    let mut buffer = [0u8; 128];
    let mut real_path = String::from("");
    loop {
        let len = read(fd as usize, &mut buffer);
        match len {
            -1 => {
                println!("{}: Bad file descriptor", fd);
                break;
            }
            0 => break,
            _ => {
                real_path = str::from_utf8(&buffer[0..len as usize])
                    .unwrap()
                    .to_string()
            }
        }
    }
    close(fd as usize);

    let fd = open(real_path.as_str(), RDONLY);
    if fd == -1 {
        println!("{}: No such file or directory", real_path);
    }
    loop {
        let len = read(fd as usize, &mut buffer);
        match len {
            -1 => {
                println!("{}: Bad file descriptor", fd);
                break;
            }
            0 => break,
            _ => print!("{}", str::from_utf8(&buffer[0..len as usize]).unwrap()),
        }
    }
}
