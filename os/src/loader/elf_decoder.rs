//! Elf文件解析子模块

use alloc::vec;
use alloc::vec::Vec;
use core::marker::Copy;

/// 无符号地址
#[derive(Clone, Copy, Debug)]
struct Elf64Addr(u64);

/// 无符号文件偏移量
#[derive(Clone, Copy, Debug)]
struct Elf64Off(u64);

/// 无符号半整型
#[derive(Clone, Copy, Debug)]
struct Elf64Half(u16);

/// 有符号半整型
#[derive(Clone, Copy, Debug)]
struct Elf64Shalf(i16);

/// 无符号整型
#[derive(Clone, Copy, Debug)]
struct Elf64Word(u32);

/// 有符号整型
#[derive(Clone, Copy, Debug)]
struct Elf64Sword(i32);

/// 无符号长整型
#[derive(Clone, Copy, Debug)]
struct Elf64Xword(u64);

/// 有符号长整型
#[derive(Clone, Copy, Debug)]
struct Elf64Sxword(i64);

/// Elf文件头标识字节数
const EI_NIDENT: usize = 16;

/// Elf头
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ElfHeader {
    e_ident: [u8; EI_NIDENT],
    e_type: Elf64Half,
    e_machine: Elf64Half,
    e_version: Elf64Word,
    /// Entry point virtual address
    e_entry: Elf64Addr,
    /// Program header table file offset
    e_phoff: Elf64Off,
    /// Section header table file offset
    e_shoff: Elf64Off,
    e_flags: Elf64Word,
    e_ehsize: Elf64Half,
    e_phentsize: Elf64Half,
    e_phnum: Elf64Half,
    e_shentsize: Elf64Half,
    e_shnum: Elf64Half,
    e_shstrndx: Elf64Half,
}

impl ElfHeader {
    /// 根据magic number判断该文件是否为合法Elf
    pub fn is_valid(&self) -> bool {
        self.e_ident[0] == 0x7f
            && self.e_ident[1] == 0x45
            && self.e_ident[2] == 0x4c
            && self.e_ident[3] == 0x46
    }

    /// 获取程序入口
    pub fn entry(&self) -> usize {
        self.e_entry.0 as usize
    }
}

/// 程序头表
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ProgramHeader {
    p_type: Elf64Word,
    p_flags: Elf64Word,
    /// Segment file offset
    p_offset: Elf64Off,
    /// Segment virtual address
    p_vaddr: Elf64Addr,
    /// Segment physical address
    p_paddr: Elf64Addr,
    /// Segment size in file
    p_filesz: Elf64Xword,
    /// Segment size in memory
    p_memsz: Elf64Xword,
    /// Segment alignment, file & memory
    p_align: Elf64Xword,
}

impl ProgramHeader {
    /// 该段是否需要加载
    pub fn is_load(&self) -> bool {
        self.p_type.0 == 1
    }

    /// 该段是否可读
    pub fn is_readable(&self) -> bool {
        self.p_flags.0 & (1 << 2) != 0
    }

    /// 该段是否可写
    pub fn is_writable(&self) -> bool {
        self.p_flags.0 & (1 << 1) != 0
    }

    /// 该段是否可执行
    pub fn is_executable(&self) -> bool {
        self.p_flags.0 & (1 << 0) != 0
    }

    /// 该段的虚拟地址
    pub fn vaddr(&self) -> usize {
        self.p_vaddr.0 as usize
    }

    /// 该段起始处相对文件头的位置
    pub fn offset(&self) -> usize {
        self.p_offset.0 as usize
    }

    /// 该段被载入内存后的大小
    pub fn mem_size(&self) -> usize {
        self.p_memsz.0 as usize
    }

    /// 该段在文件中的大小
    pub fn file_size(&self) -> usize {
        self.p_filesz.0 as usize
    }
}

/// 节头表
#[repr(C)]
pub struct SectionHeader {
    /// Section name, index in string tbl
    sh_name: Elf64Word,
    /// Type of section
    sh_type: Elf64Word,
    /// Miscellaneous section attributes
    sh_flags: Elf64Xword,
    /// Section virtual addr at execution
    sh_addr: Elf64Addr,
    /// Section file offset
    sh_offset: Elf64Off,
    /// Size of section in bytes
    sh_size: Elf64Xword,
    /// Index of another section
    sh_link: Elf64Word,
    /// Additional section information
    sh_info: Elf64Word,
    /// Section alignment
    sh_addralign: Elf64Xword,
    /// Entry size if section holds table
    sh_entsize: Elf64Xword,
}

/// Elf文件解析内容
pub struct ElfFile {
    pub header: ElfHeader,
    pub program_headers: Vec<ProgramHeader>,
}

impl ElfFile {
    /// 解析elf数据
    pub fn new(elf_data: &[u8]) -> Option<Self> {
        let elf_ptr = elf_data.as_ptr();
        let header = unsafe { *(elf_ptr as *const ElfHeader) };
        if !header.is_valid() {
            return None;
        }
        let mut program_headers = vec![];
        for i in 0..header.e_phnum.0 {
            let ph_offset =
                (header.e_phoff.0 as usize) + (header.e_phentsize.0 as usize) * (i as usize);
            let ph = unsafe { *(elf_ptr.offset(ph_offset as isize) as *const ProgramHeader) };
            program_headers.push(ph);
        }
        Some(Self {
            header,
            program_headers,
        })
    }
}
