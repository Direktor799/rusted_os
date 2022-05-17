//! BuddySystem堆内存分配器

use super::linked_list::LinkedList;
use crate::uninit_cell::UninitCell;
use alloc::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::cmp::{max, min};
use core::mem::size_of;
use core::ops::Deref;
use core::ptr::{null_mut, NonNull};

/// ORDER阶的BuddySystem分配器
#[derive(Debug)]
pub struct BuddySystemAllocator<const ORDER: usize> {
    free_list: [LinkedList; ORDER],
    total: usize,
    allocated: usize,
}

impl<const ORDER: usize> BuddySystemAllocator<ORDER> {
    /// 根据传入地址创建空分配器
    pub fn new(start_addr: usize, size: usize) -> Self {
        let mut allocator = Self {
            free_list: [LinkedList::new(); ORDER],
            total: 0,
            allocated: 0,
        };
        allocator.add(start_addr, start_addr + size);
        allocator
    }

    /// 向分配器中添加空闲地址
    pub fn add(&mut self, start_addr: usize, end_addr: usize) {
        // assume its size_of::<usize>() aligned
        let mut total = 0;
        let mut curr_addr = start_addr;
        while curr_addr + size_of::<usize>() <= end_addr {
            let lowbit = curr_addr & (!curr_addr + 1);
            let size = min(lowbit, prev_power_of_two(end_addr - curr_addr)); // 满足内存块对齐要求的情况下分配尽可能大块的内存
            total += size;
            self.free_list[size.trailing_zeros() as usize].push(curr_addr as *mut usize);
            curr_addr += size;
        }
        self.total += total;
    }

    /// 分配内存
    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let size = get_size(layout);
        let order = size.trailing_zeros() as usize;
        for curr_order in order..ORDER {
            // get smallest block which big enough
            if !self.free_list[curr_order].is_empty() {
                // split the block in chain
                for spliting_order in (order + 1..curr_order + 1).rev() {
                    let first_addr = self.free_list[spliting_order].pop().unwrap();
                    let second_addr =
                        (first_addr as usize + (1 << (spliting_order - 1))) as *mut usize;
                    self.free_list[spliting_order - 1].push(first_addr);
                    self.free_list[spliting_order - 1].push(second_addr);
                }
                self.allocated += size;
                return NonNull::new(self.free_list[order].pop().unwrap() as *mut u8)
                    .unwrap()
                    .as_ptr();
            }
        }
        null_mut()
    }

    /// 回收内存
    fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let size = get_size(layout);
        let order = size.trailing_zeros() as usize;
        let mut curr_addr = ptr as usize;
        // 合并内存碎片
        for curr_order in order..ORDER {
            let super_addr = curr_addr as usize & !(1 << curr_order);
            let buddy_addr = curr_addr as usize ^ (1 << curr_order);
            let mut buddy_found = false;
            // 寻亲之旅
            for addr in self.free_list[curr_order].iter_mut() {
                if addr.value() as usize == buddy_addr {
                    addr.pop();
                    buddy_found = true;
                }
            }
            // 找到Buddy则继续迭代向上合并
            if buddy_found {
                curr_addr = super_addr;
            } else {
                self.free_list[curr_order].push(curr_addr as *mut usize);
                self.allocated -= size;
                return;
            }
        }
    }
}

/// 类似next_power_of_two
fn prev_power_of_two(num: usize) -> usize {
    1 << (8 * size_of::<usize>() - num.leading_zeros() as usize - 1)
}

/// 根据内存布局计算实际分配大小
fn get_size(layout: Layout) -> usize {
    // 因为管理地址空间时，内存块的大小与对齐一致，所以分配时亦然，取其最大者
    max(
        layout.size().next_power_of_two(),
        max(layout.align(), size_of::<usize>()),
    )
}

/// 由于GlobalAlloc的限制导致无法使用mut方法，使用RefCell包装
pub struct HeapAllocator(pub RefCell<BuddySystemAllocator<32>>);

unsafe impl GlobalAlloc for UninitCell<HeapAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.borrow_mut().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.borrow_mut().dealloc(ptr, layout)
    }
}

impl Deref for HeapAllocator {
    type Target = RefCell<BuddySystemAllocator<32>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
