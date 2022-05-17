#![no_std]
#![no_main]

extern crate alloc;

#[macro_use]
extern crate user_lib;

use user_lib::console::get_char;

#[no_mangle]
fn main() -> i32 {
    let mut cur = alloc::string::String::new();
    let mut current_path = alloc::string::String::from("/");
    print!("root@rusted_os:{}# ", current_path);
    loop {
        let ch = get_char();
        print!("{}", ch);
        if ch == '\x7f' {
            if !cur.is_empty() {
                print!("\x08 \x08");
                cur.pop();
            }
            continue;
        }
        cur.push(ch);
        if ch == '\r' {
            println!("");
            let args = cur.split_whitespace().collect::<alloc::vec::Vec<_>>();
            // if args[0] == "ls" {
            //     if args.len() >= 2 {
            //         fs::ls_by_path(args[1]);
            //     } else {
            //         fs::ls_by_path(&current_path);
            //     }
            // } else if args[0] == "rm" {
            //     fs::delete_by_path(args[1]);
            // } else if args[0] == "mkdir" {
            //     fs::mkdir_by_path(args[1])
            // } else if args[0] == "touch" {
            //     fs::touch_by_path(args[1]);
            // } else if args[0] == "cd" {
            //     if fs::check_valid_by_path(args[1]) {
            //         current_path = alloc::string::String::from(args[1]);
            //     } else {
            //         println!("{}: No such file or directory", args[1]);
            //     }
            // } else {
            //     println!("Unknown command");
            // }
            cur.clear();
            print!("root@rusted_os:{}# ", current_path);
        }
    }
    0
}
