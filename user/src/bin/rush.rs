#![no_std]
#![no_main]

extern crate alloc;

extern crate user_lib;
use alloc::string::String;
use alloc::vec;
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
        let args = input.split_whitespace().collect::<Vec<_>>();
        if !args.is_empty() {
            match args[0] {
                "cd" => cd(&mut cwd, &args),
                "mkdir" => app_mkdir(&args),
                "cat" => app_cat(&args),
                "ln" => app_ln(&args),
                "readlink" => app_readlink(&args),
                "rm" => app_rm(&args),
                "rmdir" => app_rmdir(&args),
                "stat" => app_stat(&args),
                "ls" => {
                    let pid = fork();
                    if pid == 0 {
                        exec("/bin/ls");
                    } else {
                        let mut exit_code = 0;
                        waitpid(pid as usize, &mut exit_code);
                    }
                }
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
                0 => break,
                -1 => {
                    println!("{}: Bad file descriptor", fd);
                    break;
                }
                _ => print!("{}", str::from_utf8(&buffer[0..len as usize]).unwrap()),
            }
        }
        close(fd as usize);
    }
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

fn app_readlink(args: &Vec<&str>) {
    if args.len() == 1 {
        println!("missing operand");
        return;
    }
    for target in &args[1..] {
        let mut content = String::new();
        if readlink(target, &mut content) == 0 {
            println!("{}", content);
        }
    }
}

fn app_rm(args: &Vec<&str>) {
    if args.len() == 1 {
        println!("missing operand");
        return;
    }
    for target in &args[1..] {
        match unlink(target, 0) {
            0 => {}
            -1 => println!("cannot remove '{}': No such file or directory", target),
            -2 => println!("cannot remove '{}': Is a directory", target),
            _ => panic!(),
        }
    }
}

fn app_rmdir(args: &Vec<&str>) {
    if args.len() == 1 {
        println!("missing operand");
        return;
    }
    for target in &args[1..] {
        match unlink(target, AT_REMOVEDIR) {
            0 => {}
            -1 => println!("failed to remove '{}': No such file or directory", target),
            -2 => println!("failed to remove '{}': Not a directory", target),
            -3 => println!("failed to remove '{}': Directory not empty", target),
            _ => panic!(),
        }
    }
}

fn app_stat(args: &Vec<&str>) {
    if args.len() == 1 {
        println!("missing operand");
        return;
    }
    for target in &args[1..] {
        let fd = open(target, RDONLY);
        let mut stat = Stat::new();
        if fd == -1 {
            println!("cannot stat '{}': No such file or directory", target);
            continue;
        }
        match fstat(fd as usize, &mut stat) {
            0 => {}
            -1 => {
                println!("{}: Bad file descriptor", fd);
                continue;
            }
            _ => panic!(),
        }
        close(fd as usize);
        let file_type = match stat.mode as usize {
            CHR => "character special file",
            REG => "regular file",
            DIR => "directory",
            LNK => "symbolic link",
            _ => panic!("Unknown mode: {}", stat.mode),
        };
        println!("File: {}\t\tType: {}", target, file_type);
        println!("Size: {}\t\tInode: {}", stat.size, stat.ino);
    }
}

fn app_ls(args: &Vec<&str>) {
    let mut targets = vec![];
    if args.len() == 1 {
        targets.push(".");
    }
    for target in &args[1..] {
        targets.push(target);
    }
    for target in targets {
        let fd = open(target, RDONLY);
        let mut stat = Stat::new();
        if fd == -1 {
            println!("cannot access '{}': No such file or directory", target);
            continue;
        }
        match fstat(fd as usize, &mut stat) {
            0 => {}
            -1 => {
                println!("{}: Bad file descriptor", fd);
                continue;
            }
            _ => panic!(),
        }
        match stat.mode as usize {
            REG => {
                println!("{}", target);
            }
            DIR => {
                let mut buf = vec![0u8; stat.size as usize];
                read(fd as usize, &mut buf);
                for i in 2..buf.len() / DIRENT_SZ {
                    let offset = i * DIRENT_SZ;
                    let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const Dirent) };
                    let len = dirent
                        .name
                        .iter()
                        .position(|&v| v == 0)
                        .unwrap_or(dirent.name.len());
                    let name = str::from_utf8(&dirent.name[0..len]).unwrap();
                    print!("{}\t", name);
                }
                if buf.len() / DIRENT_SZ > 2 {
                    println!("");
                }
            }
            _ => panic!("Unknown mode: {}", stat.mode),
        };
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
        -1 => println!("{}: Bad file descriptor", fd),
        _ => println!("ok"),
    }
    close(fd as usize);
}
