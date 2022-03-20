use super::allocator;

#[global_allocator]
static ALLOCATTOR: allocator::allocator::Dummy = allocator::allocator::Dummy;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}

pub const KERNEL_HEAP_SIZE: usize = 0x80_0000;

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
