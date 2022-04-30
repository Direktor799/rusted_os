//! 用于堆内存管理的裸单向链表
use core::marker::Copy;
use core::{fmt, ptr};

/// 链表头，head指向第一个结点
/// 其指针即为数据
#[derive(Copy, Clone)]
pub struct LinkedList {
    head: *mut usize,
}

impl LinkedList {
    /// 创建空链表
    pub const fn new() -> LinkedList {
        LinkedList {
            head: ptr::null_mut(),
        }
    }

    /// 返回该链表是否为空
    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    /// 在链表头处添加结点
    pub fn push(&mut self, item: *mut usize) {
        unsafe {
            *item = self.head as usize;
        }
        self.head = item;
    }

    /// 弹出链表头处的结点
    pub fn pop(&mut self) -> Option<*mut usize> {
        if self.is_empty() {
            None
        } else {
            let item = self.head;
            self.head = unsafe { *item as *mut usize };
            Some(item)
        }
    }

    /// 返回链表的不可变迭代器
    pub fn iter(&self) -> Iter {
        Iter {
            curr: self.head,
            _list: self,
        }
    }

    /// 返回链表的可变迭代器
    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            prev: &mut self.head as *mut *mut usize as *mut usize,
            curr: self.head,
            _list: self,
        }
    }
}

impl fmt::Debug for LinkedList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// 链表的不可变迭代器
pub struct Iter<'a> {
    curr: *mut usize,
    // _list用于传递生命周期参数
    _list: &'a LinkedList,
}

impl<'a> Iterator for Iter<'a> {
    type Item = *mut usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            None
        } else {
            let item = self.curr;
            let next = unsafe { *item as *mut usize };
            self.curr = next;
            Some(item)
        }
    }
}

/// 用于可变迭代器的抽象ListNode
pub struct ListNode {
    prev: *mut usize,
    curr: *mut usize,
}

impl ListNode {
    pub fn pop(self) -> *mut usize {
        unsafe {
            *(self.prev) = *(self.curr);
        }
        self.curr
    }

    pub fn value(&self) -> *mut usize {
        self.curr
    }
}

/// 链表的可变迭代器
pub struct IterMut<'a> {
    prev: *mut usize,
    curr: *mut usize,
    // _list用于传递生命周期参数
    _list: &'a mut LinkedList,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = ListNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            None
        } else {
            let res = ListNode {
                prev: self.prev,
                curr: self.curr,
            };
            self.prev = self.curr;
            self.curr = unsafe { *self.curr as *mut usize };
            Some(res)
        }
    }
}

unit_test!(test_linked_list, {
    let mut linked_list = LinkedList::new();
    utest_assert!(linked_list.is_empty(), "New LinkedList is not empty!");
    let addrs: [*mut usize; 3] = [
        0x8100_0000 as *mut usize,
        0x8100_1000 as *mut usize,
        0x8100_2000 as *mut usize,
    ];
    for addr in addrs {
        linked_list.push(addr);
    }
    for (i, addr) in linked_list.iter().enumerate() {
        utest_assert!(addr == addrs[2 - i], "Iteration failed");
    }
    let mut mut_iter = linked_list.iter_mut();
    let first = mut_iter.next();
    utest_assert!(
        first.is_some() && first.unwrap().pop() == addrs[2],
        "Modify via iter_mut failed"
    );
    for i in (0..2).rev() {
        let addr = linked_list.pop();
        utest_assert!(addr.is_some() && addr.unwrap() == addrs[i], "Pop failed");
    }
    Ok("passed")
});
