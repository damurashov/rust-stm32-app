#![no_std]
#![no_main]
#![feature(core_intrinsics)]

extern crate rust_stm32;
use core::intrinsics;

static RODATA_VARIABLE: &[u8] = b"Rodata";
static mut BSS_VARIABLE: u32 = 0;
static mut DATA_VARIABLE: u32 = 1;

#[no_mangle]
pub extern "C" fn hard_fault() -> ! {
	loop{}
}

#[export_name = "main"]
fn entry() -> ! {
	let _rodata = RODATA_VARIABLE;
	let _bss = unsafe {&BSS_VARIABLE};
	let _data = unsafe {&DATA_VARIABLE};
	intrinsics::abort();
	loop {}
}
