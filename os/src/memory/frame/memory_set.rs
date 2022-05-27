//! 地址空间子模块
use super::address::*;
use super::page_table::{PageTable, R, U, W, X};
use super::segment::{MemorySegment, SegFlags};
use crate::config::{MEMORY_END_ADDR, MMIO, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE};
use crate::tools::elf_decoder::ElfFile;
use crate::tools::uninit_cell::UninitCell;
use alloc::{vec, vec::Vec};

extern "C" {
    fn text_start();
    fn trampoline_start();
    fn text_end();
    fn rodata_start();
    fn rodata_end();
    fn data_start();
    fn data_end();
    fn bss_start();
    fn bss_end();
    fn kernel_end();
}

/// 内核地址空间
pub static mut KERNEL_MEMORY_SET: UninitCell<MemorySet> = UninitCell::uninit();

/// 地址空间
#[derive(Debug)]
pub struct MemorySet {
    page_table: PageTable,
    segments: Vec<MemorySegment>,
}

impl MemorySet {
    /// 创建空地址空间
    pub fn new() -> Self {
        Self {
            page_table: PageTable::new(),
            segments: vec![],
        }
    }

    /// 创建新内核地址空间
    pub fn new_kernel() -> Self {
        // println!(".text [{:x}, {:x})", text_start as usize, text_end as usize);
        // println!(
        //     ".rodata [{:x}, {:x})",
        //     rodata_start as usize, rodata_end as usize
        // );
        // println!(".data [{:x}, {:x})", data_start as usize, data_end as usize);
        // println!(".bss [{:x}, {:x})", bss_start as usize, bss_end as usize);
        // println!("kernel end at {:x}", kernel_end as usize);
        let mut memory_set = Self::new();
        memory_set.map_trampoline();
        for vpn in VPNRange::new(
            VirtAddr(text_start as usize).vpn(),
            VirtAddr(text_end as usize).vpn(),
        ) {
            memory_set.page_table.map(vpn, PhysPageNum(vpn.0), R | X);
        }
        for vpn in VPNRange::new(
            VirtAddr(rodata_start as usize).vpn(),
            VirtAddr(rodata_end as usize).vpn(),
        ) {
            memory_set.page_table.map(vpn, PhysPageNum(vpn.0), R);
        }
        for vpn in VPNRange::new(
            VirtAddr(data_start as usize).vpn(),
            VirtAddr(data_end as usize).vpn(),
        ) {
            memory_set.page_table.map(vpn, PhysPageNum(vpn.0), R | W);
        }
        for vpn in VPNRange::new(
            VirtAddr(bss_start as usize).vpn(),
            VirtAddr(bss_end as usize).vpn(),
        ) {
            memory_set.page_table.map(vpn, PhysPageNum(vpn.0), R | W);
        }
        for vpn in VPNRange::new(
            VirtAddr(kernel_end as usize).vpn(),
            VirtAddr(MEMORY_END_ADDR).vpn(),
        ) {
            memory_set.page_table.map(vpn, PhysPageNum(vpn.0), R | W);
        }
        for pair in MMIO {
            for vpn in VPNRange::new(VirtAddr(pair.0).vpn(), VirtAddr(pair.0 + pair.1).vpn()) {
                memory_set.page_table.map(vpn, PhysPageNum(vpn.0), R | W);
            }
        }
        memory_set
    }

    /// 创建新用户程序地址空间
    /// 返回用户地址空间，用户栈地址，用户程序入口
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new();
        let elf = ElfFile::new(elf_data).expect("[kernel] Invalid elf file!");
        let mut max_end_vpn = VirtPageNum(0);
        memory_set.map_trampoline();
        for ph in elf.program_headers {
            if !ph.is_load() {
                continue;
            }
            let start_vpn = VirtAddr(ph.vaddr()).vpn();
            let end_vpn = VirtPageNum(VirtAddr(ph.vaddr() + ph.mem_size()).vpn().0 + 1);
            if end_vpn > max_end_vpn {
                max_end_vpn = end_vpn;
            }
            let mut perm = U;
            if ph.is_readable() {
                perm |= R
            }
            if ph.is_writable() {
                perm |= W
            }
            if ph.is_executable() {
                perm |= X;
            }
            memory_set.insert_segment(
                VPNRange::new(start_vpn, end_vpn),
                perm,
                Some(&elf_data[ph.offset()..ph.offset() + ph.file_size()]),
            );
        }
        let user_stack_start_vpn = VirtPageNum(max_end_vpn.0 + 1);
        let user_stack_end_vpn = VirtPageNum(user_stack_start_vpn.0 + USER_STACK_SIZE / PAGE_SIZE);
        memory_set.insert_segment(
            VPNRange::new(user_stack_start_vpn, user_stack_end_vpn),
            U | R | W,
            None,
        );
        memory_set.insert_segment(
            VPNRange::new(VirtAddr(TRAP_CONTEXT).vpn(), VirtAddr(TRAMPOLINE).vpn()),
            R | W,
            None,
        );
        (memory_set, user_stack_end_vpn.addr().0, elf.header.entry())
    }

    /// 在此地址空间中添加映射并分配物理页
    pub fn insert_segment(
        &mut self,
        vpn_range: VPNRange,
        seg_flags: SegFlags,
        data: Option<&[u8]>,
    ) {
        let segment = MemorySegment::new(vpn_range, seg_flags);
        if let Some(data) = data {
            segment.copy_data(data);
        }
        for (&vpn, frame) in &segment.data_frames {
            self.page_table.map(vpn, frame.ppn(), seg_flags);
        }
        self.segments.push(segment);
    }

    /// 从地址空间中删除指定映射
    pub fn remove_segment(&mut self, start_vpn: VirtPageNum) {
        let segment_index = self
            .segments
            .iter()
            .position(|segment| segment.vpn_range.curr_vpn == start_vpn)
            .unwrap();
        for vpn in self.segments[segment_index].vpn_range {
            self.page_table.unmap(vpn);
        }
        self.segments.remove(segment_index);
    }

    /// 映射跳板页
    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr(TRAMPOLINE).vpn(),
            PhysAddr(trampoline_start as usize).ppn(),
            R | X,
        );
    }

    /// 查找虚拟页号对应的物理页号
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PhysPageNum> {
        self.page_table.translate(vpn)
    }

    /// 获取该地址空间token（用于写入satp寄存器）
    pub fn satp_token(&self) -> usize {
        self.page_table.satp_token()
    }

    /// 切换到此地址空间
    pub fn activate(&self) {
        let satp = self.page_table.satp_token();
        unsafe {
            core::arch::asm!("csrw satp, {}", "sfence.vma", in(reg) satp);
        }
    }
}

impl Clone for MemorySet {
    fn clone(&self) -> Self {
        let mut new_memory_set = MemorySet::new();
        new_memory_set.map_trampoline();
        for segment in &self.segments {
            let mut data = Vec::new();
            for vpn in segment.vpn_range {
                data.extend_from_slice(self.translate(vpn).unwrap().get_bytes_array());
            }
            new_memory_set.insert_segment(segment.vpn_range, segment.flags, Some(&data));
            // copy data from another space
            // for vpn in segment.vpn_range {
            //     let src_ppn = self.translate(vpn).unwrap();
            //     let dst_ppn = new_memory_set.translate(vpn).unwrap();
            //     dst_ppn
            //         .get_bytes_array()
            //         .copy_from_slice(src_ppn.get_bytes_array());
            // }
        }
        new_memory_set
    }
}

pub fn init() {
    unsafe {
        KERNEL_MEMORY_SET = UninitCell::init(MemorySet::new_kernel());
        KERNEL_MEMORY_SET.activate();
    }
}
