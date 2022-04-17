use std::{error::Error};
use cc;

fn main() -> Result<(), Box<dyn Error>> {
    cc::Build::new().file("src/sync.s").compile("asm");
    println!("cargo:rerun-if-changed=src/sync.s");

    Ok(())
}
