
    .align 3
    .section .data
    .globl _app_num
_app_num:
    .quad 1
    .quad app_0_start
    .quad app_0_end

    .section .data
    .globl app_0_start
    .globl app_0_end
app_0_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/fork"
app_0_end:
