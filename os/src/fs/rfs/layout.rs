//! 磁盘布局子模块

use super::{block_cache::get_block_cache, block_dev::BlockDevice, DataBlock, BLOCK_SZ};
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter, Result};
use core::mem::size_of;

/// 换成咱八路军的曲子
const RFS_MAGIC: u32 = 0xdeadbeef;

/// 直接块数量
const INODE_DIRECT_COUNT: usize = 28;

/// 一级间接块数量
const INODE_INDIRECT1_COUNT: usize = BLOCK_SZ / size_of::<u32>();

/// 二级间接块数量
const INODE_INDIRECT2_COUNT: usize = INODE_INDIRECT1_COUNT * INODE_INDIRECT1_COUNT;

/// 仅直接块最大数量
const INODE_DIRECT_BOUND: usize = INODE_DIRECT_COUNT;

/// 仅直接块和以及间接块最大数量
const INODE_INDIRECT1_BOUND: usize = INODE_DIRECT_BOUND + INODE_INDIRECT1_COUNT;

/// 直接块、一级间接块、二级间接块总计最大数量
const INODE_INDIRECT2_BOUND: usize = INODE_INDIRECT1_BOUND + INODE_INDIRECT2_COUNT;

/// 目录项名长度限制
const NAME_LENGTH_LIMIT: usize = 27;

/// 第一个块，记录文件系统相关信息
#[repr(C)]
pub struct SuperBlock {
    magic: u32,
    pub total_blocks: u32,
    pub inode_bitmap_blocks: u32,
    pub inode_blocks: u32,
    pub data_bitmap_blocks: u32,
    pub data_blocks: u32,
}

impl Debug for SuperBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("SuperBlock")
            .field("total_blocks", &self.total_blocks)
            .field("inode_bitmap_blocks", &self.inode_bitmap_blocks)
            .field("inode_blocks", &self.inode_blocks)
            .field("data_bitmap_blocks", &self.data_bitmap_blocks)
            .field("data_blocks", &self.data_blocks)
            .finish()
    }
}

impl SuperBlock {
    /// 根据参数初始化当前超级块
    pub fn init(
        &mut self,
        total_blocks: u32,
        inode_bitmap_blocks: u32,
        inode_blocks: u32,
        data_bitmap_blocks: u32,
        data_blocks: u32,
    ) {
        *self = Self {
            magic: RFS_MAGIC,
            total_blocks,
            inode_bitmap_blocks,
            inode_blocks,
            data_bitmap_blocks,
            data_blocks,
        }
    }

    /// 判断超级块是否合法
    pub fn is_valid(&self) -> bool {
        self.magic == RFS_MAGIC
    }
}

/// Inode类型
#[derive(Clone, Copy, PartialEq)]
pub enum InodeType {
    File,
    Directory,
    SoftLink,
}

/// 间接块
type IndirectBlock = [u32; BLOCK_SZ / 4];

/// Inode
#[repr(C)]
pub struct Inode {
    pub size: u32,
    pub direct: [u32; INODE_DIRECT_COUNT],
    pub indirect1: u32,
    pub indirect2: u32,
    type_: InodeType,
}

impl Inode {
    /// 初始化当前Inode
    pub fn init(&mut self, type_: InodeType) {
        self.size = 0;
        self.direct.iter_mut().for_each(|v| *v = 0);
        self.indirect1 = 0;
        self.indirect2 = 0;
        self.type_ = type_;
    }

    /// 判断当前Inode是否为目录
    pub fn is_dir(&self) -> bool {
        self.type_ == InodeType::Directory
    }

    /// 判断当前Inode是否为文件
    pub fn is_file(&self) -> bool {
        self.type_ == InodeType::File
    }
    /// 判断当前Inode是否为link
    pub fn is_link(&self) -> bool {
        self.type_ == InodeType::SoftLink
    }

    /// 用于存储Inode数据的块数量
    pub fn data_blocks(&self) -> u32 {
        (self.size + BLOCK_SZ as u32 - 1) / BLOCK_SZ as u32
    }

    /// 用于存储Inode数据和元数据的块数量
    pub fn total_blocks(size: u32) -> u32 {
        let data_blocks = (size as usize + BLOCK_SZ - 1) / BLOCK_SZ;
        let mut total = data_blocks as usize;
        // indirect1
        if data_blocks > INODE_DIRECT_BOUND {
            total += 1;
        }
        // indirect2
        if data_blocks > INODE_INDIRECT1_BOUND {
            total += 1;
            // sub indirect1
            total += (data_blocks - INODE_INDIRECT1_BOUND + INODE_INDIRECT1_COUNT - 1)
                / INODE_INDIRECT1_COUNT;
        }
        total as u32
    }

    /// 需要扩充的块数量
    pub fn blocks_needed(&self, new_size: u32) -> u32 {
        assert!(new_size >= self.size);
        assert!(new_size <= INODE_INDIRECT2_BOUND as u32 * BLOCK_SZ as u32);
        Self::total_blocks(new_size) - Self::total_blocks(self.size)
    }

    /// 根据内部块id获取在设备上的块id
    pub fn get_block_id(&self, inner_id: u32, block_device: &Rc<dyn BlockDevice>) -> u32 {
        let inner_id = inner_id as usize;
        if inner_id < INODE_DIRECT_BOUND {
            // 直接块
            self.direct[inner_id]
        } else if inner_id < INODE_INDIRECT1_BOUND {
            // 一级间接块
            get_block_cache(self.indirect1 as usize, Rc::clone(block_device))
                .borrow()
                .read(0, |indirect_block: &IndirectBlock| {
                    indirect_block[inner_id - INODE_DIRECT_BOUND]
                })
        } else {
            // 二级间接块
            let last = inner_id - INODE_INDIRECT1_BOUND;
            let sub_indirect1 = get_block_cache(self.indirect2 as usize, Rc::clone(block_device))
                .borrow()
                .read(0, |indirect2: &IndirectBlock| {
                    indirect2[last / INODE_INDIRECT1_COUNT]
                });
            get_block_cache(sub_indirect1 as usize, Rc::clone(block_device))
                .borrow()
                .read(0, |indirect1: &IndirectBlock| {
                    indirect1[last % INODE_INDIRECT1_COUNT]
                })
        }
    }

    /// 扩充Inode管理的空间大小
    pub fn increase_size(
        &mut self,
        new_size: u32,
        new_blocks: Vec<u32>,
        block_device: &Rc<dyn BlockDevice>,
    ) {
        let mut current_blocks = self.data_blocks() as usize;
        self.size = new_size;
        let total_blocks = self.data_blocks() as usize;
        let mut new_blocks = new_blocks.into_iter();
        // 扩充直接块
        while current_blocks < total_blocks.min(INODE_DIRECT_BOUND) {
            self.direct[current_blocks as usize] = new_blocks.next().unwrap();
            current_blocks += 1;
        }
        // 判断是否需要继续扩充一级间接块
        if total_blocks <= INODE_DIRECT_BOUND {
            return;
        }
        // 若还未分配一级间接块
        if current_blocks == INODE_DIRECT_BOUND {
            self.indirect1 = new_blocks.next().unwrap();
        }
        // 扩充一级间接块
        get_block_cache(self.indirect1 as usize, Rc::clone(block_device))
            .borrow_mut()
            .modify(0, |indirect1: &mut IndirectBlock| {
                while current_blocks < total_blocks.min(INODE_INDIRECT1_BOUND) {
                    indirect1[current_blocks as usize - INODE_DIRECT_BOUND] =
                        new_blocks.next().unwrap();
                    current_blocks += 1;
                }
            });
        // 判断是否需要继续扩充二级间接块
        if total_blocks <= INODE_INDIRECT1_BOUND {
            return;
        }
        // 若还未分配二级间接块
        if current_blocks == INODE_INDIRECT1_COUNT {
            self.indirect2 = new_blocks.next().unwrap();
        }
        // 扩充二级间接块
        get_block_cache(self.indirect2 as usize, Rc::clone(block_device))
            .borrow_mut()
            .modify(0, |indirect2: &mut IndirectBlock| {
                while current_blocks < total_blocks {
                    let curr_sub_indirect1 =
                        (current_blocks - INODE_INDIRECT1_BOUND) / INODE_INDIRECT1_COUNT;
                    let curr_sub_direct =
                        (current_blocks - INODE_INDIRECT1_BOUND) % INODE_INDIRECT1_COUNT;
                    // 当前sub一级间接块的第一次写入时分配sub一级间接块
                    if curr_sub_direct == 0 {
                        indirect2[curr_sub_indirect1] = new_blocks.next().unwrap();
                    }
                    // 扩充sub一级间接块中的直接块
                    get_block_cache(
                        indirect2[curr_sub_indirect1] as usize,
                        Rc::clone(block_device),
                    )
                    .borrow_mut()
                    .modify(0, |indirect1: &mut IndirectBlock| {
                        indirect1[curr_sub_direct] = new_blocks.next().unwrap();
                    });
                    current_blocks += 1;
                }
            });
    }

    /// 减少Inode管理的空间大小
    pub fn decrease_size(&mut self, new_size: u32, block_device: &Rc<dyn BlockDevice>) -> Vec<u32> {
        let mut v: Vec<u32> = Vec::new();
        let current_blocks = self.data_blocks() as usize;
        self.size = new_size;
        let mut recycled_blocks = self.data_blocks() as usize;
        // 回收直接块
        while recycled_blocks < current_blocks.min(INODE_DIRECT_BOUND) {
            v.push(self.direct[recycled_blocks]);
            self.direct[recycled_blocks] = 0;
            recycled_blocks += 1;
        }
        // 判断是否需要继续回收一级间接块
        if current_blocks <= INODE_DIRECT_BOUND {
            return v;
        }
        // 回收一级间接块
        get_block_cache(self.indirect1 as usize, Rc::clone(block_device))
            .borrow()
            .read(0, |indirect1: &IndirectBlock| {
                while recycled_blocks < current_blocks.min(INODE_INDIRECT1_BOUND) {
                    v.push(indirect1[recycled_blocks - INODE_DIRECT_BOUND]);
                    recycled_blocks += 1;
                }
            });
        v.push(self.indirect1);
        self.indirect1 = 0;
        // 判断是否需要继续回收二级间接块
        if current_blocks <= INODE_INDIRECT1_BOUND {
            return v;
        }
        // 回收二级间接块
        get_block_cache(self.indirect2 as usize, Rc::clone(block_device))
            .borrow()
            .read(0, |indirect2: &IndirectBlock| {
                while recycled_blocks < current_blocks {
                    let curr_sub_indirect1 =
                        (recycled_blocks - INODE_INDIRECT1_BOUND) / INODE_INDIRECT1_COUNT;
                    let curr_sub_direct =
                        (recycled_blocks - INODE_INDIRECT1_BOUND) % INODE_INDIRECT1_COUNT;
                    // 当前sub一级间接块中内容第一次回收时回收sub一级间接块
                    if curr_sub_direct == 0 {
                        v.push(indirect2[curr_sub_indirect1]);
                    }
                    // 回收sub一级间接块中的直接块
                    get_block_cache(
                        indirect2[curr_sub_indirect1] as usize,
                        Rc::clone(block_device),
                    )
                    .borrow()
                    .read(0, |indirect1: &IndirectBlock| {
                        v.push(indirect1[curr_sub_direct]);
                    });
                    recycled_blocks += 1;
                }
            });
        v.push(self.indirect2);
        self.indirect2 = 0;
        v
    }
    /// 读取指定偏移处数据
    pub fn read_at(
        &self,
        offset: usize,
        buf: &mut [u8],
        block_device: &Rc<dyn BlockDevice>,
    ) -> usize {
        let mut curr_start = offset;
        let end = (offset + buf.len()).min(self.size as usize);
        if curr_start >= end {
            return 0;
        }
        // 计算内部起始块号
        let mut curr_block = curr_start / BLOCK_SZ;
        let mut read_size = 0usize;
        // 遍历区间内的每一个block
        loop {
            let curr_block_end = ((curr_start / BLOCK_SZ + 1) * BLOCK_SZ).min(end);
            let curr_block_read_size = curr_block_end - curr_start;
            let dst = &mut buf[read_size..read_size + curr_block_read_size];
            get_block_cache(
                self.get_block_id(curr_block as u32, block_device) as usize,
                Rc::clone(block_device),
            )
            .borrow()
            .read(0, |data_block: &DataBlock| {
                let src = &data_block
                    [curr_start % BLOCK_SZ..curr_start % BLOCK_SZ + curr_block_read_size];
                dst.copy_from_slice(src);
            });
            read_size += curr_block_read_size;
            // 读完跳出
            if curr_block_end == end {
                break;
            }
            curr_block += 1;
            curr_start = curr_block_end;
        }
        read_size
    }

    /// 将数据写入指定偏移处
    pub fn write_at(
        &mut self,
        offset: usize,
        buf: &[u8],
        block_device: &Rc<dyn BlockDevice>,
    ) -> usize {
        let mut curr_start = offset;
        let end = (offset + buf.len()).min(self.size as usize);
        assert!(curr_start <= end);
        // 计算内部起始块号
        let mut curr_block = curr_start / BLOCK_SZ;
        let mut write_size = 0usize;
        // 遍历区间内的每一个block
        loop {
            let curr_block_end = ((curr_start / BLOCK_SZ + 1) * BLOCK_SZ).min(end);
            let curr_block_write_size = curr_block_end - curr_start;
            get_block_cache(
                self.get_block_id(curr_block as u32, block_device) as usize,
                Rc::clone(block_device),
            )
            .borrow_mut()
            .modify(0, |data_block: &mut DataBlock| {
                let src = &buf[write_size..write_size + curr_block_write_size];
                let dst = &mut data_block
                    [curr_start % BLOCK_SZ..curr_start % BLOCK_SZ + curr_block_write_size];
                dst.copy_from_slice(src);
            });
            write_size += curr_block_write_size;
            // 写完跳出
            if curr_block_end == end {
                break;
            }
            curr_block += 1;
            curr_start = curr_block_end;
        }
        write_size
    }
}

/// 目录项
#[repr(C)]
pub struct Dirent {
    name: [u8; NAME_LENGTH_LIMIT + 1],
    inode_number: u32,
}

/// 目录项大小
pub const DIRENT_SZ: usize = size_of::<Dirent>();

impl Dirent {
    /// 创建空目录项
    pub fn empty() -> Self {
        Self {
            name: [0u8; NAME_LENGTH_LIMIT + 1],
            inode_number: 0,
        }
    }

    /// 根据参数创建新目录项
    pub fn new(name: &str, inode_number: u32) -> Self {
        let mut bytes = [0u8; NAME_LENGTH_LIMIT + 1];
        bytes[..name.len()].copy_from_slice(name.as_bytes());
        Self {
            name: bytes,
            inode_number,
        }
    }

    /// 获取目录项数据
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, DIRENT_SZ) }
    }

    /// 获取目录项可变数据
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *const _ as *mut u8, DIRENT_SZ) }
    }

    /// 获取目录项名称
    pub fn name(&self) -> &str {
        let len = (0..).find(|i| self.name[*i] == 0).unwrap();
        core::str::from_utf8(&self.name[..len]).unwrap()
    }

    /// 获取目录项Inode编号
    pub fn inode_number(&self) -> u32 {
        self.inode_number
    }
}
