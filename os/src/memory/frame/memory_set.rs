use super::address::*;
use super::page_table::{PageTable, R, U, W, X};
use super::segment::{MemorySegment, SegFlags};
use alloc::{vec, vec::Vec};

extern "C" {
    fn kernel_start();
    fn text_start();
    fn text_end();
    fn rodata_start();
    fn rodata_end();
    fn data_start();
    fn data_end();
    fn bss_start();
    fn bss_end();
    fn kernel_end();
}

pub struct MemorySet {
    page_table: PageTable,
    segments: Vec<MemorySegment>,
}

impl MemorySet {
    pub fn new() -> Self {
        Self {
            page_table: PageTable::new(),
            segments: vec![],
        }
    }

    pub fn new_kernel() -> Self {
        println!("kernel start at {:x}", kernel_start as usize);
        println!(".text [{:x}, {:x})", text_start as usize, text_end as usize);
        println!(
            ".rodata [{:x}, {:x})",
            rodata_start as usize, rodata_end as usize
        );
        println!(".data [{:x}, {:x})", data_start as usize, data_end as usize);
        println!(".bss [{:x}, {:x})", bss_start as usize, bss_end as usize);
        println!("kernel end at {:x}", kernel_end as usize);
        let mut memory_set = Self::new();
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
        memory_set
    }

    pub fn insert_segment(
        &mut self,
        start_vpn: VirtPageNum,
        end_vpn: VirtPageNum,
        seg_flags: SegFlags,
    ) {
        let vpn_range = VPNRange::new(start_vpn, end_vpn);
        let segment = MemorySegment::new(vpn_range, seg_flags);
        for (&vpn, frame) in segment.map_iter() {
            self.page_table.map(vpn, frame.ppn(), seg_flags);
        }
        self.segments.push(segment);
    }

    pub fn activate(&self) {
        let satp = self.page_table.satp_token();
        // self.test();
        unsafe {
            core::arch::asm!("csrw satp, {}", "sfence.vma", in(reg) satp);
        }
    }

    // fn test(&self) {
    //     let mut stack = vec![];
    //     stack.push((0, self.page_table.root_ppn));
    //     while !stack.is_empty() {
    //         let (base, ppn) = stack.pop().unwrap();
    //         for (i, pte) in ppn.get_pte_array().iter().enumerate() {
    //             let vpn = (base << 9) + i;
    //             if pte.flags() == 1 {
    //                 stack.push((vpn, pte.ppn()));
    //             } else if pte.flags() != 0 {
    //                 println!(
    //                     "{:?} -> {:?} {:b}",
    //                     VirtPageNum(vpn),
    //                     pte.ppn(),
    //                     pte.flags()
    //                 );
    //             }
    //         }
    //     }
    //     for vpn in VPNRange::new(
    //         VirtAddr(kernel_start as usize).vpn(),
    //         VirtAddr(kernel_end as usize).vpn(),
    //     ) {
    //         if let Some(pte) = self.page_table.translate(vpn) {
    //             println!("{:?} -> {:?}  {:b}", vpn, pte.ppn(), pte.flags());
    //         }
    //     }
    // }
}
