.syntax unified
critical_recursive: .word 0
addr_critical_recursive: .word critical_recursive

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
    ldr r2, [addr_critical_recursive, #0]
    ldr r3, [r2, #0]
    add r3, #1
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
    ldr r2, [addr_critical_recursive, #0]
    ldr r3, [r2, #0]
    sub r3, r2, #1
    cmp r3, #0
    bleq.n goback
    cpsie i
     /* Restore previous stack pointer, restore previous stack boundary, update program counter  */
goback:
    mov sp, r7
    pop {r7, pc}

    .section .text.hard_fault_trampoline
    .global hard_fault_trampoline
    .type hard_fault_trampoline, %function
hard_fault_trampoline:
    mrs r0, MSP /* Load the 1st arg - stack pointer value */
    b hard_fault /* Proceed w/ the user implementation of hardfault handler, if there's one. See "script.ld" and "lib.rs" */
