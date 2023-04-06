#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use bindgen::callbacks::{EnumVariantValue, ParseCallbacks};
use std::{
    env,
    path::PathBuf,
};

#[derive(Debug)]
struct CleanNdspyNamingCallbacks {}

impl ParseCallbacks for CleanNdspyNamingCallbacks {
    fn enum_variant_name(
        &self,
        enum_name: Option<&str>,
        original_variant_name: &str,
        _variant_value: EnumVariantValue,
    ) -> Option<String> {
        if let Some(enum_name) = enum_name {
            match enum_name {
                "PtDriverVersion" => {
                    Some(original_variant_name.trim_start_matches("k_PtDriver").trim_end_matches("Version").to_string())
                }
                "PtDspyCookedQueryValue" => Some(
                    original_variant_name
                        .trim_start_matches("PkDspyCQ")
                        .to_string(),
                ),
                "PtDspyError" => Some(
                    original_variant_name
                        .trim_start_matches("PkDspyError")
                        .to_string(),
                ),
                "PtDspyQueryType" => Some(
                    original_variant_name
                        .trim_start_matches("Pk")
                        .trim_end_matches("Query")
                        .to_string(),
                ),

                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=include/wrapper.h");

    let include_path = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("include");

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
        .parse_callbacks(Box::new(CleanNdspyNamingCallbacks {}))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Could not write bindings.");

    Ok(())
}
