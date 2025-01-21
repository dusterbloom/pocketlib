fn main() {
    // When using proc-macros, we don't need to generate scaffolding in build.rs
    println!("cargo:rerun-if-changed=src/lib.rs");
}