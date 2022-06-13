MEMORY {
	FLASH(rx) :  ORIGIN = 0x08000000, LENGTH = 16K
	SRAM(rwx) :  ORIGIN = 0x20000000, LENGTH = 4K
}

ENTRY(reset_trampoline);
EXTERN(RESET_VECTOR);
EXTERN(EXCEPTIONS);
EXTERN(stack_frame_swap_next)
EXTERN(_val_psp_initial);

/* To prevent exception handlers from corrupting PSP stack when in exception mode due to overlaps, a boundary for
exception handling stack will be used. So, PSP will be initialized w/ PSP + _msp_area_size  */

_msp_initial = ORIGIN(SRAM) + LENGTH(SRAM);  /* Initial value for the MSP stack pointer */
_msp_area_size = 0x200;  /* Number of bytes for exception handling stack (pointed to by MSP stack pointer) */
_psp_initial = _msp_initial - _msp_area_size;

SECTIONS {
	.vector_table ORIGIN(FLASH): {
		. = ALIGN(4);
		/* Main Stack Pointer's initial value */
		LONG(_msp_initial);

		/* Do not eliminate (optimize out) .reset_vector symbol at linking stage */
		KEEP(*(.vector_table.reset_vector));

		KEEP(*(.vector_table.exceptions));
	} > FLASH

	.text : {
		. = ALIGN(4);
		*(_val_psp_initial);
		. = ALIGN(4);
		*(_scb_icsr_address);
		/* Bring Rust / ASM inter-dependent pieces of code together.
		Preventive measure against "out of range" errors. Mixing ASM code using branch-instructions (`b`) w/ Rust might
		lead to "out of range errors" during compilation, where the code that is being branched into gets located too
		far (not in the vicinity of [-2048, 2047]) from the caller. Apparently, the linker does not make such checks. */
		*(.text.reset);
		*(.text.reset_trampoline);
		*(.text.pend_sv);
		*(.text.stack_frame_swap_next);
		*(.text.hard_fault);
		*(.text.hard_fault_trampoline);
		*(.text .text.*);
	} > FLASH

	.bss : {
		. = ALIGN(4);
		_sbss = .;
		*(.bss .bss.*)
		. = ALIGN(4);
		_ebss = .;
	} > SRAM

	.rodata : {
		. = ALIGN(4);
		*(.rodata .rodata.*);
	} > FLASH

	.data : AT(ADDR(.rodata) + SIZEOF(.rodata)) {
		. = ALIGN(4);
		_sdata = .;
		*(.data .data.*);
		. = ALIGN(4);
		_edata = .;
	} > SRAM

	_sidata = LOADADDR(.data);

	/* The remaning space is where heap and stack reside */
	.heap_and_stack : {
		. = ALIGN(4);

		/* Symbol `end` is required for libc. The ARM toolchain "rustup" installs does not seem to provide standard
		libc's `malloc` calls or alternatives, so we borrow pre-compiled static `libc` and `libnosys` libraries from
		arm-none-eabi-gcc-10 package and wrap the memory managing code around it */

		PROVIDE(end = .);
	} > SRAM

	/DISCARD/ : {
		*(.ARM.exidx .ARM.exidx.*);
		libc.a ( * )
		libm.a ( * )
		libgcc.a ( * )
	}
}

PROVIDE(nmi = default_exception_handler);
PROVIDE(hard_fault = default_exception_handler);
PROVIDE(sv_call = default_exception_handler);
PROVIDE(pend_sv = default_exception_handler);
PROVIDE(sys_tick = default_exception_handler);
PROVIDE(wwdg_irq = default_exception_handler);
PROVIDE(tim14_irq = default_exception_handler);
