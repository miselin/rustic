
fn main() {
    cc::Build::new()
        .file("src/arch/i386/start.S")
        .compile("arch-stub");
    println!("cargo:rerun-if-changed=src/arch/i386/start.s");
 }
