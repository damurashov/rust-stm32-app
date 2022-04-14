use crate::{wr, rd};

pub fn configure() {
	unsafe {
		const ENABLE: usize = 0b1;
		// RCC_CR
		wr!(RCC, CR, HSION, ENABLE);  // Enable HSI
		wr!(RCC, CR, PLLON, ENABLE);  // Enable PLL, as we want to adjust the clock freq.

		while !(rd!(RCC, CR, PLLRDY) > 0 && rd!(RCC, CR, HSIRDY) > 0) {}  // Busy-wait until the peripherals are on

		// RCC_CFGR
		const SYSTEM_CLOCK_PLL: usize = 0b10;  // select PLL as system clock
		wr!(RCC, CFGR, SW, SYSTEM_CLOCK_PLL);
		const AHB_PRESCALER_NO_DIV: usize = 0;  // AHB (HCLK) prescaler - do not curtail the frequency
		wr!(RCC, CFGR, HPRE, AHB_PRESCALER_NO_DIV);
		const APB_PRESCALER_NO_DIV: usize = 0;  // APB (PCLK) prescaler - do not curtail the frequency
		wr!(RCC, CFGR, PPRE, APB_PRESCALER_NO_DIV);
		const PLL_SOURCE_HSI_DIV_2: usize = 0;  // Select HSI/2 as input clock source
		wr!(RCC, CFGR, PLLSRC, PLL_SOURCE_HSI_DIV_2);
		const PLL_MULT_8: usize = 0b0100;  //  Multiply PLL by 12 (HSI=8(Standard) / 2(HSI/2) * 12(PLLMUL)) = 48. PLL input must not exceed 48MHz
		wr!(RCC, CFGR, PLLMUL, PLL_MULT_8);
		// RCC_AHBENR
		wr!(RCC, AHBENR, GPIOAEN, ENABLE);  // Enable GPIOA port (where USART1 resides)
		wr!(RCC, AHBENR, GPIOBEN, ENABLE);
		// RCC_APB2ENR
		wr!(RCC, APB2ENR, USART1EN, ENABLE);
		// RCC_CFGR3
		const USART1_CLOCK_SOURCE_PCK: usize = 0b00;
		wr!(RCC, CFGR3, USART1SW, USART1_CLOCK_SOURCE_PCK);  // Use PCLK as clock source for UART1
		// Debug module clock - enable
		wr!(RCC, APB2ENR, DBGMCUEN, ENABLE);
	}
}
