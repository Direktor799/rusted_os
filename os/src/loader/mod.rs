mod app_manager;
mod elf_decoder;

pub use app_manager::APP_MANAGER;
pub use elf_decoder::ElfFile;

/// init loader subsystem
pub fn init() {
    unsafe {
        APP_MANAGER.borrow_mut().init();
        // APP_MANAGER.borrow_mut().print_app_info();
    }
    println!("mod loader initialized!");
}
