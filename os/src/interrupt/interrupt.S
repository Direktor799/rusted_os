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

# 跳板页
    .section .text.trampoline
    .globl __interrupt
# 进入中断
# 保存 Context 并且进入 Rust 中的中断处理函数 interrupt::handler::handle_interrupt()
__interrupt:
    # 切换 sp 到 Context 地址, 在sscratch中保存用户栈地址
    csrrw sp, sscratch, sp
    # 保存通用寄存器，除了 x0（固定为 0）
    SAVE    x1, 1
    # 保存其他寄存器
    .set    n, 3
    .rept   29
        SAVE_N  %n
        .set    n, n + 1
    .endr
    # 取出 CSR 并保存
    csrr    t1, sstatus
    SAVE    t1, 32
    csrr    t2, sepc
    SAVE    t2, 33
    # 保存用户栈地址
    csrr    t3, sscratch
    SAVE    t3, 2
    # 加载kernel的token
    LOAD    t1, 34
    # 加载trap_handler
    LOAD    t2, 36
    # 加载kernel的sp
    LOAD    sp, 35
    # mv      a0, sp                              # context: &mut Context
    # csrr    a1, scause                          # scause: Scause
    # csrr    a2, stval                           # stval: usize

    csrw satp, t1
    sfence.vma
    jr t2

    .globl __restore
# 离开中断
# 从 Context 中恢复所有寄存器，并跳转至 Context 中 sepc 的位置
# a0: 用户空间Context  a1: 用户空间token
__restore:
    # 切换到用户空间
    csrw satp, a1
    sfence.vma
    # 保存 Context 地址到 sscratch
    csrw sscratch, a0
    # 以 Context 地址为 sp, 恢复 CSR
    mv sp, a0
    LOAD    t1, 32
    csrw    sstatus, t1
    LOAD    t2, 33
    csrw    sepc, t2

    # 恢复通用寄存器
    LOAD    x1, 1
    # 恢复 x3 至 x31
    .set    n, 3
    .rept   29
        LOAD_N  %n
        .set    n, n + 1
    .endr
    # 切换到用户栈(Context.x[2])
    LOAD sp, 2
    sret