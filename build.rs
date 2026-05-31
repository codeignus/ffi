use std::env;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let header = manifest.join("types.h");
    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let out_file = out.join("ffi_types_bindings.rs");

    // Re-generate Rust ABI bindings only when the single SoT changes.
    println!("cargo:rerun-if-changed={}", header.display());

    let bindings = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .allowlist_type("ffi_.*")
        .derive_default(true)
        // Guardrail: generate C-vs-Rust ABI layout tests from the header.
        .layout_tests(true)
        .generate()
        .expect("failed to generate ffi bindings from types.h");

    bindings
        .write_to_file(&out_file)
        .expect("failed to write ffi bindings");
}
