
    .align 3
    .section .data
    .globl _app_num
_app_num:
    .quad 2
    .quad app_0_start
    .quad app_1_start
    .quad app_1_end

    .section .data
    .globl app_0_start
    .globl app_0_end
app_0_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ls"
app_0_end:

    .section .data
    .globl app_1_start
    .globl app_1_end
app_1_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/rush"
app_1_end:
