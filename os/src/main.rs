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
    #[cfg(test)]
    test_main();
    println!("[kernel] Hello rusted_os!");
    memory::init();
    interrupt::init();
    loader::init();
    task::init();
    fs::init();
    let device = alloc::sync::Arc::new(drivers::virtio_block::VirtIOBlock::new());
    fs::efs::EasyFileSystem::format(device.clone(), 4096, 1);
    let fs = fs::efs::EasyFileSystem::open(device.clone());
    let root_inode = fs::efs::EasyFileSystem::root_inode(&fs);
    root_inode.create("a");
    root_inode.create("b");
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
