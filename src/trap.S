    .section .text
    .global _start_trap
    .align 4

_start_trap:
    csrrw   sp, mscratch, sp
    bnez    sp, 1f
    /* from M level, load sp */
    csrrw   sp, mscratch, zero
1:
    addi    sp, sp, -28 * REGBYTES
    STORE   ra, 0
    STORE   t0, 1
    STORE   t1, 2
    STORE   t2, 3
    STORE   t3, 4
    STORE   t4, 5
    STORE   t5, 6
    STORE   t6, 7
    STORE   a0, 8
    STORE   a1, 9
    STORE   a2, 10
    STORE   a3, 11
    STORE   a4, 12
    STORE   a5, 13
    STORE   a6, 14
    STORE   a7, 15
    STORE   s0, 16
    STORE   s1, 17
    STORE   s2, 18
    STORE   s3, 19
    STORE   s4, 20
    STORE   s5, 21
    STORE   s6, 22
    STORE   s7, 23
    STORE   s8, 24
    STORE   s9, 25
    STORE   s10, 26
    STORE   s11, 27
    csrr    t0, satp
    STORE   t0, 28
    mv      a0, sp
    call    _start_trap_rust
    LOAD    ra, 0
    LOAD    t0, 1
    LOAD    t1, 2
    LOAD    t2, 3
    LOAD    t3, 4
    LOAD    t4, 5
    LOAD    t5, 6
    LOAD    t6, 7
    LOAD    a0, 8
    LOAD    a1, 9
    LOAD    a2, 10
    LOAD    a3, 11
    LOAD    a4, 12
    LOAD    a5, 13
    LOAD    a6, 14
    LOAD    a7, 15
    LOAD    s0, 16
    LOAD    s1, 17
    LOAD    s2, 18
    LOAD    s3, 19
    LOAD    s4, 20
    LOAD    s5, 21
    LOAD    s6, 22
    LOAD    s7, 23
    LOAD    s8, 24
    LOAD    s9, 25
    LOAD    s10, 26
    LOAD    s11, 27
    addi    sp, sp, 28 * REGBYTES
.align 2
    csrrw   sp, mscratch, sp
    mret