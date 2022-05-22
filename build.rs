use std::{error::Error, env};
use cc;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rustc-link-arg=-L{}/lib", env::current_dir().unwrap().to_str().unwrap());
    cc::Build::new().file("src/thread/sync.s").file("src/thread/task.s").compile("asm");
    println!("cargo:rerun-if-changed=src/thread/sync.s");
    println!("cargo:rerun-if-changed=src/thread/task.s");

    println!("cargo:rustc-link-arg=-mcpu=cortex-m0");
    println!("cargo:rustc-link-arg=-mthumb");
    println!("cargo:rustc-link-arg=-Tscript.ld");
    println!("cargo:rustc-link-arg=-Wl,-lc_nano");
    println!("cargo:rustc-link-arg=-Wl,-lnosys");

    Ok(())
}
