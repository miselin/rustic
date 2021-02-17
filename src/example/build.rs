
fn main() {
    // TODO: pick linker scripts based on architecture
    println!("cargo:rustc-link-arg-bins=src/linker.ld");
 }
