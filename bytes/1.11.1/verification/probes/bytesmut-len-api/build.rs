use std::{env, process::Command};

fn main() {
    println!("cargo:rustc-check-cfg=cfg(bytes_original_unique_gate)");
    println!("cargo:rustc-cfg=bytes_original_unique_gate");
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    assert!(Command::new("python3")
        .arg(format!("{manifest}/extract_bytes_mut.py"))
        .status()
        .unwrap()
        .success());
    println!("cargo:rerun-if-changed=extract_bytes_mut.py");
    println!("cargo:rerun-if-changed=../../../src/bytes_mut.rs");
    println!("cargo:rerun-if-changed=../../../src/ownership_proof/raw_vec.rs");
    println!("cargo:rerun-if-changed=../../../src/ownership_proof/owned_region.rs");
    println!("cargo:rerun-if-changed=../../../src/ownership_proof/view_region.rs");
    println!("cargo:rerun-if-changed=../../../src/provenance_specs.rs");
}
