#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use alloc::{string::String, vec};
use core::iter::Iterator;
use core::str;
use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    let procs_fd = open("/proc", RDONLY);
    let mut stat = Stat::new();
    fstat(procs_fd as usize, &mut stat);
    let mut buf = vec![0u8; stat.size as usize];
    read(procs_fd as usize, &mut buf);
    close(procs_fd as usize);
    let procs_info = (2..buf.len() / DIRENT_SZ)
        .into_iter()
        .map(|i| {
            let offset = i * DIRENT_SZ;
            let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const Dirent) };
            let len = dirent
                .name
                .iter()
                .position(|&v| v == 0)
                .unwrap_or(dirent.name.len());
            let pid = str::from_utf8(&dirent.name[0..len]).unwrap().to_owned();
            let mem_fd = open(&(String::from("/proc/") + &pid + "/mem"), RDONLY);
            let mut stat = Stat::new();
            fstat(mem_fd as usize, &mut stat);
            let mut buf = vec![0u8; stat.size as usize];
            read(mem_fd as usize, &mut buf);
            close(mem_fd as usize);
            let mem = str::from_utf8(&buf).unwrap().to_owned();

            let cmd_fd = open(&(String::from("/proc/") + &pid + "/cmd"), RDONLY);
            let mut stat = Stat::new();
            fstat(cmd_fd as usize, &mut stat);
            let mut buf = vec![0u8; stat.size as usize];
            read(cmd_fd as usize, &mut buf);
            close(cmd_fd as usize);
            let cmd = str::from_utf8(&buf).unwrap().to_owned();

            (pid, mem, cmd)
        })
        .collect::<Vec<_>>();
    println!("PID\tMEM\t\tCMD");
    procs_info.into_iter().for_each(|(pid, mem, cmd)| {
        println!("{}\t{}\t\t{}", pid, mem, cmd);
    });
    0
}
