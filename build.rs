// build.rs
extern crate bindgen;

use std::env;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wrapper = "include/wrapper.h";

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed={}", wrapper);

    let (include_path, lib_path) = match &env::var("DELIGHT") {
        Err(_) => {
            eprintln!("Building against 3Delight 2.1.2");

            let lib_path = PathBuf::from(&env::var("OUT_DIR")?);

            #[cfg(feature = "download_3delight_lib")]
            {
                use std::io::Write;

                #[cfg(target_os = "windows")]
                let lib = "https://www.dropbox.com/s/9iavkggor0ecc1x/3Delight.dll";
                #[cfg(target_os = "macos")]
                let lib = "https://www.dropbox.com/s/7vle92kcqbbyn8o/lib3delight.dylib";
                #[cfg(target_os = "linux")]
                let lib = "https://www.dropbox.com/s/hr62te8yg1d2e36/lib3delight.so";

                let lib_file_path = lib_path.join(Path::new(lib).file_name().unwrap());

                if !lib_file_path.exists() {
                    // Download the libs to build against.
                    // We do not care of this fails (yet)
                    // as this is only needed when the
                    // crate is linked against.
                    (|| {
                        let lib_data = reqwest::blocking::get(lib).ok()?.bytes().ok()?;
                        std::fs::File::create(lib_file_path)
                            .ok()?
                            .write_all(&lib_data)
                            .ok()?;
                        Some(())
                    })();
                }
            }

            (
                PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("include"),
                lib_path,
            )
        }
        Ok(path) => {
            eprintln!("Building against locally installed 3Delight @ {}", &path);
            let delight = Path::new(&path);

            (delight.join("include"), delight.join("lib"))
        }
    };

    eprintln!("include: {}", include_path.display());
    eprintln!("lib:     {}", lib_path.display());

    // Emit linker searchpath
    println!("cargo:rustc-link-search={}", lib_path.display());
    // Link to lib3delight
    println!("cargo:rustc-link-lib=dylib=3delight");

    // Build bindings
    let bindings = bindgen::Builder::default()
        .header(wrapper)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Searchpath
        .clang_arg(format!("-I{}", include_path.display()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("ndspy-bindings.rs"))
        .expect("Couldn't write bindings.");

    Ok(())
}
