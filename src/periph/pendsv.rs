use crate::{wr, rd};

pub fn configure() {
	use crate::reg::*;
	unsafe {
		const SCB_SHP_1_PRI14_POS: usize = 16;
		const SCB_SHP_1_PRI14_MSK: usize = 0xff << SCB_SHP_1_PRI14_POS;
		const PENDSV_PRIO: usize  = 0xf0;  // Set the minimum priority to PendSV (bits [0:3] are `r` only, no need to use those)
		wr!(SCB, SHP_1, PRI14, PENDSV_PRIO);
	}
}
