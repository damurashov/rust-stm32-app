use crate::{reg, regop, wr, rd};

pub fn configure() {
	use reg::*;
	// The baudrate is calculated based on the assumption that the system clock's frequency is 48 MHz.
	const SYSTEM_CLOCK_FREQ: usize = 48_000_000;
	const BAUDRATE: usize = 57_600;
	unsafe {
		wr!(USART, 1, BRR, SYSTEM_CLOCK_FREQ / BAUDRATE);  // Set baudrate
		wr!(USART, 1, CR1, RE, 1);  // Usart, enable receiver
		wr!(USART, 1, CR1, TE, 1);  // Usart, enable transmitter
		wr!(USART, 1, CR1, UE, 1);  // Usart, enable
	}
}

pub fn read(buf: &mut [u8]) {
	use reg::*;
	unsafe {
		for c in buf {
			while !(rd!(USART, "1", ISR, RXNE) != 1) {}  // Wait for the read-ready bit
			*c = rd!(USART, "1", RDR) as u8;
		}
	}
}

pub fn write(buf: &[u8]) {
	use reg::*;
	unsafe {
		for c in buf {
			wr!(USART, "1", TDR, *c as usize);
			while rd!(USART, "1", ISR, TC) != 1 {}  // Wait until written
			wr!(USART, "1", ICR, TCCF, 1);  // Clear transfer-complete bit
		}
	}
}
