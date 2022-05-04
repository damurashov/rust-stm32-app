#![no_std]
#![no_main]
#![feature(core_intrinsics)]

mod periph;
mod reg;
#[macro_use] mod thread;
#[macro_use] mod regop;
mod mem;

extern crate rust_stm32;
use core::intrinsics;
use core::alloc::{Layout, GlobalAlloc};

static RODATA_VARIABLE: &[u8] = b"Rodata";
static mut BSS_VARIABLE: u32 = 0;
static mut DATA_VARIABLE: u32 = 1;

#[no_mangle]
pub fn hard_fault(_sp: *const u32) -> ! {
	loop{}
}

#[no_mangle]
pub fn tim14_irq() {
	loop {}
}

static mut COUNTER: u32 = 0;

#[no_mangle]
pub fn sys_tick() {
	unsafe {
		COUNTER = (COUNTER + 1) % 1000;

		if COUNTER == 0 {
			periph::usart::write("Hello".as_bytes());
		}
	}
}

#[export_name = "main"]
fn entry() -> ! {
	use crate::thread::sync::*;

	let _rodata = RODATA_VARIABLE;
	let _bss = unsafe {&BSS_VARIABLE};
	let _data = unsafe {&DATA_VARIABLE};
	periph::rcc::configure();
	periph::gpio::configure();
	periph::usart::configure();
	periph::systick::configure();
	periph::tim14::configure(100000);
	periph::tim14::set_timeout(10000);

	let mut a: u32 = 1;

	unsafe {
		let mut _mem = crate::mem::ALLOCATOR.alloc(Layout::from_size_align_unchecked(42, 1));
	}

	critical!({
		a = 42;
	});

	loop {}
}
