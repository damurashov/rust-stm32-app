use core::{arch::asm, intrinsics};

extern "C" {
	pub fn critical_enter();
	pub fn critical_exit();
}

#[macro_export]
macro_rules! critical {
	($code:block) => {
		unsafe {
			critical_enter();
		}

		$code

		unsafe {
			critical_exit();
		}
	}
}

/// RAII wrapper over critical section invoke
pub struct Critical {}

impl Critical {
	fn new() -> Critical {
		unsafe {
			critical_enter();
		}
		Critical {}
	}
}

impl Drop for Critical {
	fn drop(&mut self) {
		unsafe {
			critical_exit();
		}
	}
}

pub trait Lock {
	fn lock(&mut self);
	fn try_lock(&mut self) -> bool;
	fn unlock(&mut self);
	fn check_locked(&self) -> bool;
}

pub struct Sem {
	free: u8,
	max: u8,
}

impl Sem {
	pub fn new(free: u8, max: u8) -> Sem {
		if free > max {
			intrinsics::abort();
		}
		Self {free, max}
	}
}

impl Lock for Sem {
	fn try_lock(&mut self) -> bool {
		let mut ret: bool = false;
		let _critical = Critical::new();

		if !self.free > 0 {
			self.free -= 1;
			ret = true;
		}

		ret
	}

	fn lock(&mut self) {
		if !self.try_lock() {
			while !self.try_lock() {
				unsafe {
					asm!{"wfe"};
				}
			}
		}
	}

	fn unlock(&mut self) {
		let _critical = Critical::new();

		if self.free < self.max {
			self.free += 1;
			unsafe {
				asm!{"sev"};
			}
		}
	}

	fn check_locked(&self) -> bool {
		let _critical = Critical::new();

		self.free == 0
	}
}
