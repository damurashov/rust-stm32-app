use crate::periph::usart;
pub use core::fmt;

pub struct UartLogger;

impl fmt::Write for UartLogger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        usart::write(s.as_bytes());
        Ok(())
    }
}
