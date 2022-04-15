///
/// \param value    - value to be written into the register
/// \param register - address of the register to write to
///
pub unsafe fn write(val: usize, register: usize) {
	*(register as *mut usize) = val;
}

///
/// \param value    - value to be written into the register
/// \param register - address of the register to write to
/// \param offset   - position of the register
///
pub unsafe fn write_mask(val: usize, register: usize, mask: usize) {
	let reg = &mut *(register as *mut usize);
	*reg = *reg & !mask;
	*reg |= (val << mask.trailing_zeros()) & mask;
}

///
/// \param register - address of the register to read
///
/// \return value of the register
///
pub unsafe fn read(register: usize) -> usize {
	*(register as *const usize)
}

///
/// \param register - address of the register to read
/// \param offset   - offset where the value is stored at
/// \param mask     - mask to extract the value
///
/// \return value of the register
///
pub unsafe fn read_mask(register: usize, mask: usize) -> usize {
	return (*(register as *const usize) & mask) >> mask.trailing_zeros();
}


#[macro_export]
macro_rules! rd {
	($group:ident, $($id:literal ,)? $reg:ident, $fragment:ident) => {
		{
			use paste::paste;

			let base: usize = paste!{ [<$group $($id)? _ BASE>] };
			let offset: usize = paste!{ [<$group _ $reg _ OFFSET>] };
			let mask: usize = paste!{[<$group _ $reg _ $fragment _ MSK>]};
			let pos: usize = paste!{[<$group _ $reg _ $fragment _ POS>]};

			(*((base + offset) as *const usize) & mask) >> pos
		}
	};
	($group:ident, $($id:literal ,)? $reg:ident) => {
		{
			use paste::paste;

			let base: usize = paste!{ [<$group $($id)? _ BASE>] };
			let offset: usize = paste!{ [<$group _ $reg _ OFFSET>] };

			*((base + offset) as *const usize)
		}
	};
}

#[macro_export]
macro_rules! wr {
	($group:ident, $($id:literal ,)? $reg:ident, $fragment:ident, $val:expr) => {
		{
			use paste::paste;

			let base: usize = paste!{ [<$group $($id)? _ BASE>] };
			let offset: usize = paste!{ [<$group _ $reg _ OFFSET>] };
			let mask: usize = paste!{[<$group _ $reg _ $fragment _ MSK>]};
			let pos: usize = paste!{[<$group _ $reg _ $fragment _ POS>]};
			let chunk_reset: usize = *((base + offset) as *mut usize) & !mask;

			*((base + offset) as *mut usize) = chunk_reset | (mask & ($val << pos));
		}
	};
	($group:ident, $($id:literal ,)? $reg:ident, $val:expr) => {
		{
			use paste::paste;

			let base: usize = paste!{ [<$group $($id)? _BASE>] };
			let offset: usize = paste!{ [<$group _ $reg _ OFFSET>] };

			*((base + offset) as *mut usize) = $val;
		}
	}
}
