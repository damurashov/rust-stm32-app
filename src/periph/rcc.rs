use crate::{wu32, ru32};

pub fn configure() {
    const ENABLE: u32 = 0b1;
    // RCC_CR
    wu32!(RCC, CR, HSION, ENABLE);  // Enable HSI
    wu32!(RCC, CR, PLLON, ENABLE);  // Enable PLL, as we want to adjust the clock freq.

    while !(ru32!(RCC, CR, PLLRDY) > 0 && ru32!(RCC, CR, HSIRDY) > 0) {}  // Busy-wait until the peripherals are on

    // RCC_CFGR
    const SYSTEM_CLOCK_PLL: u32 = 0b10;  // select PLL as system clock
    wu32!(RCC, CFGR, SW, SYSTEM_CLOCK_PLL);
    const AHB_PRESCALER_NO_DIV: u32 = 0;  // AHB (HCLK) prescaler - do not curtail the frequency
    wu32!(RCC, CFGR, HPRE, AHB_PRESCALER_NO_DIV);
    const APB_PRESCALER_NO_DIV: u32 = 0;  // APB (PCLK) prescaler - do not curtail the frequency
    wu32!(RCC, CFGR, PPRE, APB_PRESCALER_NO_DIV);
    const PLL_SOURCE_HSI_DIV_2: u32 = 0;  // Select HSI/2 as input clock source
    wu32!(RCC, CFGR, PLLSRC, PLL_SOURCE_HSI_DIV_2);
    const PLL_MULT_12: u32 = 0b1010;  //  Multiply PLL by 12 (HSI=8(Standard) / 2(HSI/2) * 12(PLLMUL)) = 48. PLL input must not exceed 48MHz
    const PLL_MULT_8: u32 = 0b0100;  //  Multiply PLL by 12 (HSI=8(Standard) / 2(HSI/2) * 12(PLLMUL)) = 48. PLL input must not exceed 48MHz
    wu32!(RCC, CFGR, PLLMUL, PLL_MULT_8);
    // RCC_AHBENR
    wu32!(RCC, AHBENR, GPIOAEN, ENABLE);  // Enable GPIOA port (where USART1 resides)
    // RCC_APB2ENR
    wu32!(RCC, APB2ENR, USART1EN, ENABLE);
    // RCC_CFGR3
    const USART1_CLOCK_SOURCE_PCK: u32 = 0b00;
    wu32!(RCC, CFGR3, USART1SW, USART1_CLOCK_SOURCE_PCK);
}
