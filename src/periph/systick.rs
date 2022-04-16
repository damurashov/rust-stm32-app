use crate::{wr, rd};

pub fn configure() {
    use crate::reg::*;

    unsafe {
        wr!(SYSTICK, LOAD, RELOAD, 4_000_000 / 1000 - 1);  // A SysTick request is required every 1ms. The clock line is divided by 8, so 32 / 8 = 4 MHz. Clock frequency is 32MHz.
        wr!(SYSTICK, VAL, CURRENT, 0);  // Initialize current value
        wr!(SYSTICK, CTRL, TICKINT, 1);  // Enable SysTick exception request
        wr!(SYSTICK, CTRL, ENABLE, 1);  // Enable SysTick counter
    }
}
