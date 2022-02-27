#![no_std]
#![no_main]

extern crate rust_stm32;

#[export_name = "main"]
fn entry() -> ! {
    loop {}
}
