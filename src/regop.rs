type TargetUint = u32;

///
/// \param value    - value to be written into the register
/// \param register - address of the register to write to
///
unsafe fn write(val: TargetUint, register: TargetUint) {
    *(register as *mut TargetUint) = val;
}

///
/// \param value    - value to be written into the register
/// \param register - address of the register to write to
/// \param offset   - position of the register
///
unsafe fn write_at(val: TargetUint, register: TargetUint, offset: TargetUint) {
    *(register as *mut TargetUint) = val << offset;
}

///
/// \param register - address of the register to read
///
/// \return value of the register
///
unsafe fn read(register: TargetUint) -> TargetUint {
    *(register as *const TargetUint)
}

///
/// \param register - address of the register to read
/// \param offset   - offset where the value is stored at
/// \param mask     - mask to extract the value
///
/// \return value of the register
///
unsafe fn read_at(register: TargetUint, offset: TargetUint, mask: TargetUint) -> TargetUint {
    (*(register as *const TargetUint) & mask) >> offset
}
