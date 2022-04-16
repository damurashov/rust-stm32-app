#![no_std]
#![no_main]
#![feature(core_intrinsics)]

mod periph;
mod reg;
#[macro_use] mod regop;
extern crate rust_stm32;
use core::intrinsics;

static RODATA_VARIABLE: &[u8] = b"Rodata";
static mut BSS_VARIABLE: u32 = 0;
static mut DATA_VARIABLE: u32 = 1;

#[no_mangle]
pub fn hard_fault(_sp: *const u32) -> ! {
	loop{}
}

#[no_mangle]
pub fn sys_tick() {
	loop {}
}

#[export_name = "main"]
fn entry() -> ! {
	let _rodata = RODATA_VARIABLE;
	let _bss = unsafe {&BSS_VARIABLE};
	let _data = unsafe {&DATA_VARIABLE};
	periph::rcc::configure();
	periph::gpio::configure();
	periph::usart::configure();
	periph::systick::configure();

	let greetings = "Hello";
	loop {
		periph::usart::write(greetings.as_bytes());
	}
}
