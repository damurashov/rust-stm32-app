.syntax unified

.global get_stack_frame_next
.global get_stack_frame_current

/* Info for clearing PendSV "interrupt pending" bit */
_scb_icsr_reg_address: .word 0xE000ED04
_scb_icsr_pendsvclr: .word (1 << 27)

	.section .text.pend_sv
	.global pend_sv
	.type pend_sv, %function
/* Handles task switching, relying partially on Rust-provided wrappers over task-managing code */
pend_sv:
	/* The LD contains EXC_RETURN */
	push {lr}
	/* Pop EXC_RETURN, thus endicating end of handler routine */
	pop {pc}
