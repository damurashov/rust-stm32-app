.syntax unified
.cpu cortex-m0
.fpu softvfp
.thumb

@ .section .rodata._scb_icsr_address
@ .global _scb_icsr_address
/* Info for clearing PendSV "interrupt pending" bit */
.align 4
_scb_icsr_address: .word 0xE000ED04
.align 4
_scb_icsr_pendsvclr: .word (1 << 27)

	.section .text.pend_sv
	.global pend_sv
	.type pend_sv, %function
	.align 4
/* Handles task switching, relying partially on Rust-provided wrappers over task-managing code */
pend_sv:
	/* The lr contains EXC_RETURN */
	push {lr}
	/* Disable "interrupt pending" bit (by setting "clear pending") */
	ldr r1, =_scb_icsr_address
	ldr r1, [r1]
	ldr r3, [r1]
	movs r2, #1
	lsls r2, #27
	orrs r3, r2
	str r3, [r1]
	/* Store current registers */
	push {r4-r7}
	mov r1, r8
	push {r1}
	mov r1, r9
	push {r1}
	mov r1, r10
	push {r1}
	mov r1, r11
	push {r1}
	/* Switch context */
	mrs r0, PSP
	mrs r1, MSP
	bl stack_frame_swap_next
	/* Restore current registers */
	pop {r1}
	mov r11, r1
	pop {r1}
	mov r10, r1
	pop {r1}
	mov r9, r1
	pop {r1}
	mov r8, r1
	pop {r4-r7}
	/* Pop EXC_RETURN, thus endicating end of handler routine */
	pop {pc}