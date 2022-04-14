use crate::{regop, reg};

pub fn configure() {
	// GPIOB, PB8, Open-drain
	const GPIO_MODER_OUTPUT: usize = 0b01;  // General purpose IO
	const GPIO_MODER_ALTERNATE_FUNCTION: usize = 0b10;  // General purpose IO
	const GPIO_OTYPER_OPENDRAIN: usize = 0b1;  // Output type - open-drain
	const GPIO_OSPEEDR_HIGH: usize = 0b11;  // High speed output
	const GPIO_PUPDR_NOPULL: usize = 0b00;  // No pull-up, no pull-down
	const PA2_AF_USART1_TX: usize = 1;
	const PA3_AF_USART1_RX: usize = 1;

	// CubeMX deviates from its naming scheme here, so everything has to be set manually
	unsafe {
		// LED
		regop::write_mask(GPIO_MODER_OUTPUT, reg::GPIOA_BASE + reg::GPIO_MODER_OFFSET, reg::GPIO_MODER_MODER4_MSK);
		regop::write_mask(GPIO_OTYPER_OPENDRAIN, reg::GPIOA_BASE + reg::GPIO_OTYPER_OFFSET, reg::GPIO_OTYPER_OT_4);
		regop::write_mask(GPIO_OSPEEDR_HIGH, reg::GPIOA_BASE + reg::GPIO_OSPEEDR_OFFSET, reg::GPIO_OSPEEDR_OSPEEDR4_MSK);
		regop::write_mask(GPIO_PUPDR_NOPULL, reg::GPIOA_BASE + reg::GPIO_PUPDR_OFFSET, reg::GPIO_PUPDR_PUPDR4_MSK);
		regop::write_mask(0, reg::GPIOA_BASE + reg::GPIO_BSRR_OFFSET, reg::GPIO_BSRR_BS_4);
		// USART 1
		regop::write_mask(GPIO_MODER_ALTERNATE_FUNCTION, reg::GPIOA_BASE + reg::GPIO_MODER_OFFSET, reg::GPIO_MODER_MODER2_MSK);  // AF mode, PA2 - TX
		regop::write_mask(GPIO_MODER_ALTERNATE_FUNCTION, reg::GPIOA_BASE + reg::GPIO_MODER_OFFSET, reg::GPIO_MODER_MODER3_MSK);  // AF mode, PA3 - RX
		regop::write_mask(PA2_AF_USART1_TX, reg::GPIOA_BASE + reg::GPIO_AFR_OFFSET, reg::GPIO_AFRL_AFRL2_MSK);  // PA2, alternative function # 1
		regop::write_mask(PA3_AF_USART1_RX, reg::GPIOA_BASE + reg::GPIO_AFR_OFFSET, reg::GPIO_AFRL_AFRL3_MSK);  // PA3, alternative function # 1
	}
}
