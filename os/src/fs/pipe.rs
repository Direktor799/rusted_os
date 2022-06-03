//! 管道
use super::File;
use crate::{fs::EOT, memory::frame::user_buffer::UserBuffer};
use alloc::rc::{Rc, Weak};
use core::cell::RefCell;

use crate::task::suspend_current_and_run_next;

pub struct Pipe {
    readable: bool,
    writable: bool,
    buffer: Rc<RefCell<PipeRingBuffer>>,
}

impl Pipe {
    pub fn read_end_with_buffer(buffer: Rc<RefCell<PipeRingBuffer>>) -> Self {
        Self {
            readable: true,
            writable: false,
            buffer,
        }
    }
    pub fn write_end_with_buffer(buffer: Rc<RefCell<PipeRingBuffer>>) -> Self {
        Self {
            readable: false,
            writable: true,
            buffer,
        }
    }
}
/// 管道缓冲区大小
const RING_BUFFER_SIZE: usize = 32;

#[derive(Copy, Clone, PartialEq)]
/// 管道缓冲区状态
enum RingBufferStatus {
    Full,
    Empty,
    Normal,
}

pub struct PipeRingBuffer {
    arr: [u8; RING_BUFFER_SIZE],
    head: usize,
    tail: usize,
    status: RingBufferStatus,
    // 通过此字段判断管道所有写端是否都已经被关闭
    write_end: Weak<Pipe>,
}

/// 对管道缓冲区的操作
impl PipeRingBuffer {
    pub fn new() -> Self {
        Self {
            arr: [0; RING_BUFFER_SIZE],
            head: 0,
            tail: 0,
            status: RingBufferStatus::Empty,
            write_end: Weak::new(),
        }
    }
    /// 返回write_end的weak指针
    pub fn set_write_end(&mut self, write_end: &Rc<Pipe>) {
        self.write_end = Rc::downgrade(write_end);
    }
    pub fn write_byte(&mut self, byte: u8) {
        self.status = RingBufferStatus::Normal;
        self.arr[self.tail] = byte;
        self.tail = (self.tail + 1) % RING_BUFFER_SIZE;
        if self.tail == self.head {
            self.status = RingBufferStatus::Full;
        }
    }
    pub fn read_byte(&mut self) -> u8 {
        self.status = RingBufferStatus::Normal;
        let c = self.arr[self.head];
        self.head = (self.head + 1) % RING_BUFFER_SIZE;
        if self.head == self.tail {
            self.status = RingBufferStatus::Empty;
        }
        c
    }
    /// 判断管道是否可读
    pub fn available_read(&self) -> usize {
        if self.status == RingBufferStatus::Empty {
            0
        } else if self.tail > self.head {
            self.tail - self.head
        } else {
            self.tail + RING_BUFFER_SIZE - self.head
        }
    }
    /// 判断管道是否可写
    pub fn available_write(&self) -> usize {
        if self.status == RingBufferStatus::Full {
            0
        } else {
            RING_BUFFER_SIZE - self.available_read()
        }
    }
    /// 判断管道是否所有写端都已关闭
    pub fn all_write_ends_closed(&self) -> bool {
        self.write_end.upgrade().is_none()
    }
}

/// Return (read_end, write_end)
pub fn make_pipe() -> (Rc<Pipe>, Rc<Pipe>) {
    let buffer = Rc::new(RefCell::new(PipeRingBuffer::new()));
    let read_end = Rc::new(Pipe::read_end_with_buffer(buffer.clone()));
    let write_end = Rc::new(Pipe::write_end_with_buffer(buffer.clone()));
    buffer.borrow_mut().set_write_end(&write_end);
    (read_end, write_end)
}

impl File for Pipe {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    // 从管道中读
    fn read(&self, buf: UserBuffer) -> usize {
        assert!(self.readable());
        let mut buf_iter = buf.into_iter();
        let mut read_size = 0usize;
        loop {
            let mut ring_buffer = self.buffer.borrow_mut();
            let loop_read = ring_buffer.available_read();
            if loop_read == 0 {
                if ring_buffer.all_write_ends_closed() {
                    if let Some(byte_ref) = buf_iter.next() {
                        *byte_ref = EOT as u8;
                        read_size += 1;
                    }
                    return read_size;
                }
                drop(ring_buffer);
                suspend_current_and_run_next();
                continue;
            }
            // read at most loop_read bytes
            for _ in 0..loop_read {
                if let Some(byte_ref) = buf_iter.next() {
                    *byte_ref = ring_buffer.read_byte();
                    read_size += 1;
                } else {
                    return read_size;
                }
            }
        }
    }
    /// 向管道中写
    fn write(&self, buf: UserBuffer) -> usize {
        assert!(self.writable());
        let mut buf_iter = buf.into_iter();
        let mut write_size = 0usize;
        loop {
            let mut ring_buffer = self.buffer.borrow_mut();
            let loop_write = ring_buffer.available_write();
            if loop_write == 0 {
                drop(ring_buffer);
                suspend_current_and_run_next();
                continue;
            }
            // write at most loop_write bytes
            for _ in 0..loop_write {
                if let Some(byte_ref) = buf_iter.next() {
                    ring_buffer.write_byte(*byte_ref);
                    write_size += 1;
                } else {
                    return write_size;
                }
            }
        }
    }
}
