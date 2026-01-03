use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../../sai/SAI/inc");

    let sai_include_path =
        env::var("SAI_INCLUDE_PATH").unwrap_or_else(|_| "../../sai/SAI/inc".to_string());

    // Tell cargo to look for shared libraries in the specified directory
    if let Ok(lib_path) = env::var("SAI_LIBRARY_PATH") {
        println!("cargo:rustc-link-search=native={}", lib_path);
    }

    // Generate bindings for SAI headers
    // Note: We include individual headers instead of sai.h to avoid experimental dependencies
    let bindings = bindgen::Builder::default()
        // Core SAI headers
        .header(format!("{}/saitypes.h", sai_include_path))
        .header(format!("{}/saistatus.h", sai_include_path))
        // API headers for L2 switching
        .header(format!("{}/saiswitch.h", sai_include_path))
        .header(format!("{}/saiport.h", sai_include_path))
        .header(format!("{}/saivlan.h", sai_include_path))
        .header(format!("{}/saifdb.h", sai_include_path))
        .header(format!("{}/sailag.h", sai_include_path))
        .header(format!("{}/saibridge.h", sai_include_path))
        // Include directory
        .clang_arg(format!("-I{}", sai_include_path))
        // Generate comments from headers
        .generate_comments(true)
        // Derive common traits
        .derive_default(true)
        .derive_debug(true)
        .derive_copy(true)
        .derive_eq(true)
        .derive_partialeq(true)
        // Allowlist for SAI types
        .allowlist_type("sai_.*")
        .allowlist_type("_sai_.*")
        .allowlist_function("sai_.*")
        .allowlist_var("SAI_.*")
        .allowlist_var("_SAI_.*")
        // Prepend enum names to avoid conflicts
        .prepend_enum_name(false)
        // Translate macro constants
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Use libc types
        .ctypes_prefix("libc")
        .generate()
        .expect("Unable to generate SAI bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("sai_bindings.rs"))
        .expect("Couldn't write bindings");

    println!("cargo:rustc-link-lib=dylib=sai");
}
