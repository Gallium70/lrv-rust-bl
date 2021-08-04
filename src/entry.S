    .section .text.entry
    .global entry_point
entry_point:
    j do_start
    .align 2
magic_num:
    .word 0xdeadbeef

    .global external_dtb
    .align 3
external_dtb:
    .fill 0x4000

    .align 3
do_start:
    li x1, 0
    li x2, 0
    li x3, 0
    li x4, 0
    li x5, 0
    li x6, 0
    li x7, 0
    li x8, 0
    li x9, 0
// save a0 and a1; arguments from previous boot loader stage:
//  li x10, 0
//  li x11, 0
    li x12, 0
    li x13, 0
    li x14, 0
    li x15, 0
    li x16, 0
    li x17, 0
    li x18, 0
    li x19, 0
    li x20, 0
    li x21, 0
    li x22, 0
    li x23, 0
    li x24, 0
    li x25, 0
    li x26, 0
    li x27, 0
    li x28, 0
    li x29, 0
    li x30, 0
    li x31, 0
    csrwi mscratch, 0
    csrwi mip, 0
    csrwi mie, 0
    csrwi mideleg, 0
    csrwi medeleg, 0
    csrr    a2, mhartid
    lui     t0, %hi(_max_hart_id)
    add     t0, t0, %lo(_max_hart_id)
    bgtu    a2, t0, _start_abort
    la      sp, _stack_start
    lui     t0, %hi(_hart_stack_size)
    add     t0, t0, %lo(_hart_stack_size)
.ifdef __riscv_mul
    mul     t0, a2, t0
.else
    beqz    a2, 2f  // Jump if single-hart
    mv      t1, a2
    mv      t2, t0
1:
    add     t0, t0, t2
    addi    t1, t1, -1
    bnez    t1, 1b
2:
.endif
    sub     sp, sp, t0
    csrw    mscratch, zero

    j       main

_start_abort:
    wfi
    j _start_abort