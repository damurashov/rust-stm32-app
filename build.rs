use std::{error::Error};
use cc;

fn main() -> Result<(), Box<dyn Error>> {
    cc::Build::new().file("src/thread/sync.s").file("src/thread/task.s").compile("asm");
    println!("cargo:rerun-if-changed=src/thread/sync.s");
    println!("cargo:rerun-if-changed=src/thread/task.s");

    Ok(())
}
