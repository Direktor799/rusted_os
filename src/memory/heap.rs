use super::allocator::allocator::OutsideBuddySystemAllocator;

pub const KERNEL_HEAP_SIZE: usize = 0x80_0000;

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

#[global_allocator]
static mut ALLOCATOR: OutsideBuddySystemAllocator<32> = OutsideBuddySystemAllocator::<32>::new();

pub fn init() {
    unsafe {
        ALLOCATOR
            .borrow_mut()
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}
