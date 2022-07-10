#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(ptr_to_from_bits)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(generic_const_exprs)]
#![feature(int_roundings)]

mod periph;
mod reg;
#[macro_use] mod thread;
#[macro_use] mod regop;
mod mem;
mod tim;
mod init;
#[macro_use] mod log;

use core::intrinsics;
use core::alloc::{Layout, GlobalAlloc};
use core::fmt::Write;

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
	unsafe {
		wr!(TIM, "14", SR, UIF, 0);  // Clear interrupt flag, so it will not request interrupts indefinitely
        wr!(SCB, ICSR, PENDSVSET, 1);  // Trigger PendSV interrupt for context switching
	}
}

static mut COUNTER: u32 = 0;

#[no_mangle]
pub fn sys_tick() {
	unsafe {
		COUNTER += 1;
	}
}

fn task() {
	loop {
		periph::usart::write("I am a task".as_bytes());
	}
}

#[export_name = "main"]
fn entry() -> ! {
	use crate::log::Logger;

	periph::rcc::configure();
	periph::gpio::configure();
	periph::usart::configure();
	periph::pendsv::configure();

	const TIM14_RESOLUTION_HZ: usize = 500;
	periph::tim14::configure(TIM14_RESOLUTION_HZ);
	periph::tim14::set_timeout(tim::Duration::Milliseconds(3000));

	log!("Hi there!");

	let stack = thread::task::StaticAlloc::<512>::new();
	let mut task = thread::task::Task::from_rs(task, stack.into());

	match task.start() {
		Err(_) => {log!("Something went wrong");}
		Ok(_) => {},
	};

	loop {}
}
