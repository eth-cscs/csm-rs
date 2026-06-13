//! Build-time codegen of the HSM HTTP client from the committed
//! OpenAPI 3.0 spec at `src/hsm/csm_api_docs.openapi3.json`.
//!
//! Output is `$OUT_DIR/hsm_generated.rs`, included from
//! `src/hsm/generated.rs`. Re-runs when the JSON changes.

use std::{env, fs, path::PathBuf};

fn main() {
    let spec_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src/hsm/csm_api_docs.openapi3.json");
    println!("cargo:rerun-if-changed={}", spec_path.display());
    println!("cargo:rerun-if-changed=build.rs");

    let src = fs::read_to_string(&spec_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", spec_path.display()));
    let spec: openapiv3::OpenAPI =
        serde_json::from_str(&src).expect("csm_api_docs.openapi3.json is not valid JSON");

    let mut generator = progenitor::Generator::default();
    let tokens = generator
        .generate_tokens(&spec)
        .expect("progenitor codegen failed; re-run `make convert-spec` and check the JSON");
    let ast: syn::File = syn::parse2(tokens).expect("generated tokens do not parse");
    let pretty = prettyplease::unparse(&ast);

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("hsm_generated.rs");
    fs::write(&out, pretty).unwrap_or_else(|e| panic!("write {}: {e}", out.display()));
}
