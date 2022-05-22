use std::{error::Error, env};
use cc;

fn main() -> Result<(), Box<dyn Error>> {
    cc::Build::new().file("src/thread/sync.s").file("src/thread/task.s").compile("asm");
    println!("cargo:rerun-if-changed=src/thread/sync.s");
    println!("cargo:rerun-if-changed=src/thread/task.s");

    println!("cargo:rustc-link-search={}/lib/", env::current_dir().unwrap().to_str().unwrap());
    println!("cargo:rustc-link-lib=nosys");
    println!("cargo:rustc-link-lib=c_nano");

    Ok(())
}
