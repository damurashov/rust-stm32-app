#![no_std]
#![no_main]
#![feature(core_intrinsics)]

mod periph;
mod reg;
#[macro_use] mod thread;
#[macro_use] mod regop;
mod mem;
mod tim;

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
	use reg::*;
	use crate::wr;
	unsafe {
		wr!(TIM, "14", SR, UIF, 0);  // Clear interrupt flag, so it will not request interrupts indefinitely
        wr!(SCB, ICSR, PENDSVSET, 1);  // Trigger PendSV interrupt for context switching
	}
	periph::usart::write("Hello".as_bytes());
}

static mut COUNTER: u32 = 0;

#[no_mangle]
pub fn sys_tick() {
	unsafe {
		COUNTER += 1;
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

	const TIM14_RESOLUTION_HZ: usize = 500;
	periph::tim14::configure(TIM14_RESOLUTION_HZ);
	periph::tim14::set_timeout(tim::Duration::Milliseconds(500));
	periph::pendsv::configure();

	let mut a: u32 = 1;

	unsafe {
		let mut _mem = crate::mem::ALLOCATOR.alloc(Layout::from_size_align_unchecked(42, 1));
	}

	critical!({
		a = 42;
	});

	loop {}
}
