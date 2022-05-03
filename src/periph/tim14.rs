use crate::{wr, periph::rcc, reg::*};

static mut RESOLUTION : usize = 0;

pub fn set_timeout(microseconds: usize) {
	unsafe {
		let arr = RESOLUTION * microseconds / 1_000_000;
		wr!(TIM, "14", ARR, ARR, arr);  // Update the auto-reload register
		wr!(TIM, "14", EGR, UG, 1);  // Trigger UEV event to reset the counter
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
		wr!(TIM, "14", CR1, ARPE, 1);  // Enable preload for the Auto-reload register's value (ARR), so there is no need to wait for `UEV` event to get it transferred (preloaded) into its shadow register
		wr!(TIM, "14", CR1, UDIS, 0);  // Do not disable UEV generation. So on counter overflow or UG bit setting, a UEV will be generated, and the timer's counter will be reset
		wr!(TIM, "14", DIER, UIE, 1);  // Enable interrupt on UIE
		wr!(TIM, "14", PSC, PSC, psc_value);

		wr!(TIM, "14", CR1, CEN, 1);  // Counter ENable
	}
}
