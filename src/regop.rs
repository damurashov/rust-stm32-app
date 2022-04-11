unsafe fn write(val: u32, register: u32) {
    *(register as *mut u32) = val;
}

unsafe fn write_off(val: u32, register: u32, offset: u32) {
    *(register as *mut u32) = val << offset;
}
