    .section .text
    .globl __timer_traps
    .align 2
__timer_traps:
    csrrw sp, mscratch, sp
    sd t0, 0 * 8(sp)
    sd t1, 1 * 8(sp)
    sd t2, 2 * 8(sp)

    ld t0, 3 * 8(sp)
    ld t1, 4 * 8(sp)
    ld t2, 0(t0)
    add t2, t1, t2
    sd t2, 0(t0)

    li t0, 2
    csrw sip, t0

    ld t0, 0 * 8(sp)
    ld t1, 1 * 8(sp)
    ld t2, 2 * 8(sp)
    csrrw sp, mscratch, sp

    mret