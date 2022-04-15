use crate::{reg, regop};

pub fn configure() {
	// The baudrate is calculated based on the assumption that the system clock's frequency is 48 MHz.
	const SYSTEM_CLOCK_FREQ: usize = 48_000_000;
	const BAUDRATE: usize = 57_600;
	unsafe {
		regop::write(BAUDRATE, reg::USART1_BASE + reg::USART_BRR_OFFSET);  // Set baudrate
		regop::write_mask(1, reg::USART1_BASE + reg::USART_CR1_OFFSET, reg::USART_CR1_RE_MSK);  // Usart, enable receiver
		regop::write_mask(1, reg::USART1_BASE + reg::USART_CR1_OFFSET, reg::USART_CR1_TE_MSK);  // Usart, enable transmitter
		regop::write_mask(1, reg::USART1_BASE + reg::USART_CR1_OFFSET, reg::USART_CR1_UE_MSK);  // Usart, enable
	}
}

pub fn read(buf: &mut [u8]) {
	unsafe {
		for c in buf {
			while !(regop::read_mask(reg::USART1_BASE + reg::USART_ISR_OFFSET, reg::USART_ISR_RXNE_MSK) != 1) {}  // Wait for the read-ready bit
			*c = regop::read(reg::USART1_BASE + reg::USART_RDR_OFFSET) as u8;  // Read from the register. Clear the bit as a side effect.
		}
	}
}

pub fn write(buf: &[u8]) {
	unsafe {
		for c in buf {
			regop::write((*c).into(), reg::USART1_BASE + reg::USART_TDR_OFFSET);
		}
	}
}
