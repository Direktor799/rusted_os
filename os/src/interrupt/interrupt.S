# altmacro 使得 SAVE_N %n 成为可能
.altmacro
# 设置寄存器大小与中断上下文大小（32个通用寄存器+2个需要保存的CSR）
.set    REG_SIZE, 8
.set    CONTEXT_SIZE, 34

.align 2

# 用于将寄存器的值保存到栈指定位置的宏
.macro SAVE reg, offset
    sd  \reg, \offset*8(sp)
.endm

# 用于批量将寄存器的值保存到栈指定位置的宏
.macro SAVE_N n
    SAVE  x\n, \n
.endm

# 用于从栈指定位置读取恢复到寄存器的宏
.macro LOAD reg, offset
    ld  \reg, \offset*8(sp)
.endm

# 用于批量从栈指定位置读取恢复到寄存器的宏
.macro LOAD_N n
    LOAD  x\n, \n
.endm

    .section .text
    .globl __interrupt
# 进入中断
# 保存 Context 并且进入 Rust 中的中断处理函数 interrupt::handler::handle_interrupt()
__interrupt:
    csrrw sp, sscratch, sp                      # 切换到内核栈
    addi    sp, sp, -CONTEXT_SIZE*REG_SIZE      # 在栈上开辟 Context 所需的空间
    SAVE    x1, 1                               # 保存通用寄存器，除了 x0（固定为 0）
    .set    n, 3                                # 保存其他寄存器
    .rept   29
        SAVE_N  %n
        .set    n, n + 1
    .endr
    csrr    t1, sstatus                         # 取出 CSR 并保存
    SAVE    t1, 32
    csrr    t2, sepc
    SAVE    t2, 33
    csrr    t3, sscratch                        # 将原来的 sp(x2)写入 2 位置
    SAVE    t3, 2
    mv      a0, sp                              # context: &mut Context
    csrr    a1, scause                          # scause: Scause
    csrr    a2, stval                           # stval: usize
    jal  interrupt_handler

    .globl __restore
# 离开中断
# 从 Context 中恢复所有寄存器，并跳转至 Context 中 sepc 的位置
__restore:
    mv sp, a0                                   # 中断恢复 + U初始化
    # 恢复 CSR
    LOAD    t1, 32
    csrw    sstatus, t1
    LOAD    t2, 33
    csrw    sepc, t2
    LOAD    t3, 2
    csrw    sscratch, t3

    # 恢复通用寄存器
    LOAD    x1, 1
    # 恢复 x3 至 x31
    .set    n, 3
    .rept   29
        LOAD_N  %n
        .set    n, n + 1
    .endr
    addi    sp, sp, CONTEXT_SIZE*REG_SIZE       # 归还内核栈空间
    csrrw sp, sscratch, sp                      # 切换到用户栈
    sret