    .section .text.entry
    .globl _start
# 程序入口处 分配栈空间并调用rust_main
_start:
    la sp, boot_stack_top
    call rust_main

    .section .bss.stack
    .globl boot_stack
boot_stack:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
