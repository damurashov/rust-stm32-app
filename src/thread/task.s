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

	.align 4
	.type _memcpy, %function
/* Copy data from one location to another
 r0 - dest
 r1 - src
 r2 - count
*/
_memcpy:
	push {lr}
	add r2, r0  @ Set reg. for boundary check
	b cond_check

copy_inc:
	ldm r1!, {r3}  @ Load from source
	stm r0!, {r3}  @ Push to destination

cond_check:
	cmp r0, r2  @ Boundary check
	bcc copy_inc
	pop {pc}

	.section .text.pend_sv
	.global pend_sv
	.type pend_sv, %function
	.align 4
/* Handles task switching, relying partially on Rust-provided wrappers over task-managing code */
pend_sv:
	/* The lr contains EXC_RETURN */
	push {lr}
	/* Disable "interrupt pending" bit (by setting "clear pending") */
	ldr r1, =_scb_icsr_address @ Values r0-r3 (and some others) are already saved in the stack
	ldr r1, [r1]
	ldr r3, [r1]
	movs r2, #1
	lsls r2, #27
	orrs r3, r2
	str r3, [r1]

	@ Get stack frame addresses to swap
	bl task_frame_switch_get_swap
	cmp r1, #0
	beq pend_sv_exit @ Swap is not required, as there is no currently running task
	cmp r0, r1
	beq pend_sv_exit  @ Swap is not required, as there is only one running task

	@ Store current registers. See `StackFrameLayout` in `task.rs`
	mov r3, r11
	push {r3}
	mov r3, r10
	push {r3}
	mov r3, r9
	push {r3}
	mov r3, r8
	push {r3}
	push {r4-r7}
	mrs r3, PSP
	push {r3}

	@ Save the current stack frame, if there is one
	cmp r0, #0
	beq stack_frame_load_next  @ There is no current task, proceed to loading the context of the next one
	push {r1}  @ r1 (next stack frame address) will be used latter
	@ Copy 8 automatically saved registers from PSP to the current stack frame
	mrs r1, PSP
	movs r2, #32
	bl _memcpy
	@ Copy the rest 9 registers from the MSP stack (r0, destiation, retains an accumulated address value, see `_memcpy` for details)
	add r1, sp, #4
	movs r2, #36
	bl _memcpy
	@ Restore r1 (next stack frame address)
	pop {r1}

stack_frame_load_next:
	mrs r0, PSP  @ Set destination for _memcpy
	@ Copy the first 8 automatically saved registers from the next stack frame to PSP
	movs r2, #32
	bl _memcpy
	@ Copy the remaining 9 registers from the next stack frame to MSP (r1, source, retains an accumulated address value, see `_memcpy` for details)
	mrs r0, MSP
	movs r2, #36
	bl _memcpy  @ Swap the current stack

	@ Pop current registers from the stack (by that moment, the stack has been changed)
	pop {r0}
	msr PSP, r0
	pop {r4-r7}
	pop {r0}
	mov r8, r0
	pop {r0}
	mov r9, r0
	pop {r0}
	mov r10, r0
	pop {r0}
	mov r11, r0

pend_sv_exit:
	@ Pop EXC_RETURN, thus endicating end of handler routine
	pop {pc}
