.syntax unified
.section .bss.critical_recursive
critical_recursive: .word 0x0

    .section .text.critical_enter
    .global critical_enter
    .type critical_enter, %function
critical_enter:
    /* push stack, update return value */
    push {r7, lr}  @ r7 - stack lower boundary, lr - next instruction.
    add r7, sp, #0
    /* Disable interrupts */
    cpsid i
    /* Apply memory barriers */
    dsb
    isb
    /* Increase recursive flag */
    ldr r2, =critical_recursive
    ldr r3, [r2, #0]
    adds r3, r3, #1
    str r3, [r2, #0]
    /* Restore previous stack pointer, restore previous stack boundary, update program counter  */
    mov sp, r7
    pop {r7, pc}

    .section .text.critical_exit
    .global critical_exit
    .type critical_exit, %function
critical_exit:
    /* push stack, update return value */
    push {r7, lr}  @ r7 - stack lower boundary, lr - next instruction.
    add r7, sp, #0
    /* Decrease recursive flag */
    ldr r2, =critical_recursive
    ldr r3, [r2, #0]
    subs r3, r3, #1
    cmp r3, #0
    bgt critical_exit_ret
    cpsie i
critical_exit_ret:
     /* Restore previous stack pointer, restore previous stack boundary, update program counter  */
    str r3, [r2, #0]
    mov sp, r7
    pop {r7, pc}
