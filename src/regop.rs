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
	let reg= &mut *(register as *mut usize);
	*reg = (*reg & !mask) & ((val << mask.trailing_zeros()) & mask);
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
