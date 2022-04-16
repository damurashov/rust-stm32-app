use crate::{wr, rd};

pub fn configure() {
    use crate::reg::*;

    unsafe {
        wr!(SYSTICK, LOAD, RELOAD, 32_000_000 / 1000 - 1);  // Clock frequency is 32MHz. A SysTick request is required every 1ms
        wr!(SYSTICK, VAL, CURRENT, 0);  // Initialize current value
        wr!(SYSTICK, CTRL, TICKINT, 1);  // Enable SysTick exception request
        wr!(SYSTICK, CTRL, ENABLE, 1);  // Enable SysTick counter
    }
}
