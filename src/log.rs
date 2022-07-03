use crate::periph::usart;
use core::fmt;
use core::fmt::Write;

pub struct UartLogger;

impl fmt::Write for UartLogger {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		usart::write(s.as_bytes());
		Ok(())
	}
}

pub use UartLogger as Logger;

#[macro_export]
macro_rules! log {
	($format:expr $(, $p:expr)*) => {
		write!(Logger{}, $format $(, $p)*)
	};
}

pub extern "C" fn log_arr(arr: *const usize, size: usize) {
	unsafe {
		for i in 0..size as isize {
			log!("{} {} \r\n", i, *arr.offset(i));
		}

		log!("\r\n");
	}
}
