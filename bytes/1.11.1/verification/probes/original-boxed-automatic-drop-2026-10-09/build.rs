fn main() {
    println!("cargo:rustc-check-cfg=cfg(bytes_original_constructor_gate)");
    if std::env::var_os("CARGO_FEATURE_PUBLIC_CONSTRUCTOR").is_some() {
        println!("cargo:rustc-cfg=bytes_original_constructor_gate");
        assert!(std::process::Command::new("python3")
            .arg("extract_constructor.py")
            .status()
            .unwrap()
            .success());
    }
    for path in [
        "../../../src/bytes.rs",
        "../../../src/bytes/bytes_record.rs",
        "../../../src/bytes/vtable_record.rs",
        "../../../src/bytes_mut.rs",
        "../../../src/ownership_proof/raw_vec.rs",
        "extract_constructor.py",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }
}
