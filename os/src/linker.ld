OUTPUT_ARCH(riscv)

ENTRY(_start)

BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;

    kernel_start = .;
    . = ALIGN(4K);
    text_start = .;

    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    . = ALIGN(4K);
    rodata_start = .;

    .rodata : {
        *(.rodata .rodata.*)
    }
    . = ALIGN(4K);
    data_start = .;

    .data : {
        *(.data .data.*)
    }
    . = ALIGN(4K);
    bss_start = .;

    .bss : {
        *(.sbss .bss .bss.*)
    }
    . = ALIGN(4K);
    kernel_end = .;
}