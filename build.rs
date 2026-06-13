//! Build-time codegen of the HSM, CFS, BSS, BOS, and PCS HTTP clients from
//! their respective OpenAPI specs.
//!
//! Each call to `generate_one` reads one spec file, runs progenitor on
//! it, pretty-prints the result, and writes it under `$OUT_DIR`. The
//! `src/<module>/generated.rs` files `include!` the corresponding
//! output. Re-runs when either spec changes.

use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // HSM: OpenAPI 3.0 JSON (converted from the upstream Swagger 2.0).
    generate_one(
        &manifest_dir.join("src/hsm/csm_api_docs.openapi3.json"),
        &out_dir.join("hsm_generated.rs"),
        SpecFormat::Json,
    );

    // CFS: OpenAPI 3.0.2 YAML (upstream-tracked directly).
    generate_one(
        &manifest_dir.join("src/cfs/csm_api_docs.yaml"),
        &out_dir.join("cfs_generated.rs"),
        SpecFormat::Yaml,
    );

    // BSS: OpenAPI 3.0 JSON (converted from the upstream Swagger 2.0).
    generate_one(
        &manifest_dir.join("src/bss/csm_api_docs.openapi3.json"),
        &out_dir.join("bss_generated.rs"),
        SpecFormat::Json,
    );

    // BOS: OpenAPI 3.0.3 YAML (upstream-tracked directly).
    generate_one(
        &manifest_dir.join("src/bos/csm_api_docs.yaml"),
        &out_dir.join("bos_generated.rs"),
        SpecFormat::Yaml,
    );

    // PCS: OpenAPI 3.0.0 YAML (upstream-tracked directly).
    generate_one(
        &manifest_dir.join("src/pcs/csm_api_docs.yaml"),
        &out_dir.join("pcs_generated.rs"),
        SpecFormat::Yaml,
    );
}

enum SpecFormat {
    Json,
    Yaml,
}

fn generate_one(spec_path: &PathBuf, out_path: &PathBuf, format: SpecFormat) {
    println!("cargo:rerun-if-changed={}", spec_path.display());

    let src = fs::read_to_string(spec_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", spec_path.display()));

    let spec: openapiv3::OpenAPI = match format {
        SpecFormat::Json => serde_json::from_str(&src)
            .unwrap_or_else(|e| panic!("parse {} as JSON: {e}", spec_path.display())),
        SpecFormat::Yaml => serde_yaml::from_str(&src)
            .unwrap_or_else(|e| panic!("parse {} as YAML: {e}", spec_path.display())),
    };

    let mut generator = progenitor::Generator::default();
    let tokens = generator
        .generate_tokens(&spec)
        .unwrap_or_else(|e| {
            panic!(
                "progenitor codegen failed for {}: {e}. \
                 Check the spec is valid OpenAPI 3.0 and contains no \
                 progenitor-unsupported constructs.",
                spec_path.display()
            )
        });
    let ast: syn::File =
        syn::parse2(tokens).expect("generated tokens do not parse");
    let pretty = prettyplease::unparse(&ast);

    fs::write(out_path, pretty)
        .unwrap_or_else(|e| panic!("write {}: {e}", out_path.display()));
}
