
    .align 3
    .section .data
    .globl _app_num
_app_num:
    .quad 4
    .quad app_0_start
    .quad app_1_start
    .quad app_2_start
    .quad app_3_start
    .quad app_3_end

    .section .data
    .globl app_0_start
    .globl app_0_end
app_0_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/00ones"
app_0_end:

    .section .data
    .globl app_1_start
    .globl app_1_end
app_1_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/01twos"
app_1_end:

    .section .data
    .globl app_2_start
    .globl app_2_end
app_2_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/02threes"
app_2_end:

    .section .data
    .globl app_3_start
    .globl app_3_end
app_3_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/03yield"
app_3_end:
