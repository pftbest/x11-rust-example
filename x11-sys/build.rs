extern crate bindgen;

use std::env;
use std::path::PathBuf;
use bindgen::RustTarget;

fn main() {
    let target = env::var("TARGET").unwrap();

    if target.contains("apple") {
        // Library path for XQuartz
        println!("cargo:rustc-link-search=/opt/X11/lib");
    }

    // Link X11 libraries
    println!("cargo:rustc-link-lib=X11");
    println!("cargo:rustc-link-lib=Xext");

    // Configure bindgen
    let mut config = bindgen::Builder::default()
        .rust_target(RustTarget::Stable_1_21)
        .header("wrapper.h");

    if target.contains("apple") {
        // Add include path for XQuartz
        config = config.clang_arg("-I/opt/X11/include")
    }

    // Generate the bindings
    let bindings = config.generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
