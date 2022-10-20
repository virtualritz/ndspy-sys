#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
// build.rs
extern crate bindgen;

use std::{
    env,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: make this generic & work on Linux/Windows
    println!("cargo:rerun-if-env-changed=DELIGHT");
    println!("cargo:rerun-if-changed=include/wrapper.h");

    let include_path = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("include");

    if cfg!(feature = "link_lib3delight") {
        let delight = &env::var("DELIGHT").expect(
            "Linking against 3Delight requires a 3Delight installation and the\n\
        DELIGHT environment variable pointing to it.",
        );
        // Emit linker searchpath
        println!(
            "cargo:rustc-link-search={}",
            Path::new(delight).join("lib").display()
        );
        // Link to lib3delight
        println!("cargo:rustc-link-lib=dylib=3delight");
    }

    eprintln!("include: {}", include_path.display());

    // Build bindings
    let bindings = bindgen::Builder::default()
        .header("include/wrapper.h")
        .allowlist_function("Dspy.*")
        .allowlist_type("PtDspy.*")
        .allowlist_type("PtDriver.*")
        .allowlist_type("UserParameter")
        .allowlist_var("PkDspy.*")
        .rustified_enum(".*")
        .prepend_enum_name(false)
        // Searchpath
        .clang_arg(format!("-I{}", include_path.display()))
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Could not write bindings.");

    Ok(())
}
