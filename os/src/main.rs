#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(test::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[macro_use]
mod console;
#[macro_use]
mod test;
mod config;
mod drivers;
mod fs;
mod interrupt;
mod loader;
mod memory;
mod panic;
mod sbi;
mod sync;
mod syscall;
mod task;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

/// This is where we start.
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    memory::init();
    #[cfg(test)]
    test_main();
    println!("[kernel] Hello rusted_os!");
    interrupt::init();
    loader::init();
    task::init();
    drivers::init();
    fs::init();
    let root_inode = unsafe { fs::inode::ROOT_INODE.as_ref().unwrap() };
    root_inode.clear();
    let nodea = root_inode
        .create("a", fs::layout::InodeType::Directory)
        .unwrap();
    let nodeb = nodea.create("b", fs::layout::InodeType::File).unwrap();
    nodeb.write_at(0, "1234".as_bytes());

    let nodec = fs::inode::find_inode_by_full_path("/a/b").unwrap();
    let mut buffer = [0u8; 233];
    let len = nodec.read_at(0, &mut buffer);
    println!("{}", core::str::from_utf8(&buffer[..len]).unwrap());
    for name in root_inode.ls() {
        println!("{}", name);
    }
    root_inode.clear();
    for name in root_inode.ls() {
        println!("{}", name);
    }
    // task::run();
    panic!("Dummy as fuck");
}
