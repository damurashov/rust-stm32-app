use core::alloc::{GlobalAlloc, Layout};

extern "C" {
	fn malloc(sz: usize) -> *mut u8;
    fn free(ptr: *mut u8);
    fn calloc(num: usize, size: usize) -> *mut u8;
    fn realloc(ptr: *mut u8, size: usize) -> *mut u8;
}

pub struct Cmem();

unsafe impl GlobalAlloc for Cmem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        malloc(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        free(ptr);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        calloc(1, layout.size())
    }

    unsafe fn realloc(&self, ptr: *mut u8, _: Layout, new_size: usize) -> *mut u8 {
        realloc(ptr, new_size)
    }
}

#[global_allocator]
pub static ALLOCATOR: Cmem = Cmem();
