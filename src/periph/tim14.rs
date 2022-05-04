use crate::{wr, periph::rcc, reg::*, tim};

static mut RESOLUTION : usize = 0;

pub fn set_timeout(duration: tim::Duration) {
	unsafe {
		let arr = match duration {
			tim::Duration::Microseconds(d) => RESOLUTION * d / 1_000_000,
			tim::Duration::Milliseconds(d) => RESOLUTION * d / 1_000,
			tim::Duration::Seconds(d) => RESOLUTION * d
		};

		wr!(TIM, "14", ARR, ARR, arr);  // Update the auto-reload register
		wr!(TIM, "14", EGR, UG, 1);  // Trigger UEV event to reset the counter
		wr!(TIM, "14", CR1, CEN, 1);  // Counter ENable
	}
}

/// Configures tim14.
///
/// As it is conceived, it is supposed to trigger context switching (implemented in `PendSV` interrupt).
pub fn configure(resolution_hz: usize) {

	unsafe {
		if RESOLUTION != 0 {
			return;  // Already configured
		}
	}

	let psc_value: usize = rcc::get_clock_frequency() / resolution_hz - 1;

	unsafe {
		RESOLUTION = resolution_hz;
		wr!(RCC, APB1ENR, TIM14EN, 1);  // Enable clock for TIM 14
		wr!(TIM, "14", PSC, PSC, psc_value);
		wr!(TIM, "14", CR1, ARPE, 1);  // Enable preload for the Auto-reload register's value (ARR), so there is no need to wait for `UEV` event to get it transferred (preloaded) into its shadow register
		wr!(TIM, "14", CR1, UDIS, 0);  // Do not disable UEV generation. So on counter overflow or UG bit setting, a UEV will be generated, and the timer's counter will be reset
		wr!(TIM, "14", DIER, UIE, 1);  // Enable interrupt on UIE
		wr!(NVIC, ISER_0, 0x1 << 19);  // Enable Interrupt #19 (TIM 14 IRQ)
		wr!(TIM, "14", EGR, UG, 1);  // Trigger UEV event to reset the counter
	}
}
