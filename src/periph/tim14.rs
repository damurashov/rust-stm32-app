use crate::{wr, rd};

pub fn set_timeout(microseconds: usize) {
}

pub fn configure() {
    use crate::reg::*;

    unsafe {
        wr!(TIM, "14", CR1, ARPE, 1);  // Enable preload for the Auto-reload register's value (ARR), so there is no need to wait for `UEV` event to get it transferred (preloaded) into its shadow register
        wr!(TIM, "14", CR1, UDIS, 0);  // Do not disable UEV generation. So on counter overflow or UG bit setting, a UEV will be generated, and the timer's counter will be reset
        wr!(TIM, "14", DIER, UIE, 1);  // Enable interrupt on UIE
        wr!(TIM, "14", PSC, PSC, 320 - 1);  // Given that the system clock is 32MHz, and there is no APB1 prescaler, this will give us timer resolution of 100 KHz

        wr!(TIM, "14", CR1, CEN, 1);  // Counter ENable
    }
}
