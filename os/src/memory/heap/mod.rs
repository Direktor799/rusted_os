use crate::config::KERNEL_HEAP_SIZE;
use allocator::OutsideBuddySystemAllocator;

mod allocator;
mod linked_list;

#[global_allocator]
static mut HEAP_ALLOCATOR: OutsideBuddySystemAllocator<32> =
    OutsideBuddySystemAllocator::<32>::new();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}

static mut KERNEL_HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init() {
    unsafe {
        HEAP_ALLOCATOR
            .borrow_mut()
            .init(KERNEL_HEAP.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}
