fn main() {
    println!("cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)");
    if std::env::var_os("CARGO_FEATURE_PUBLIC_SHARED").is_some() {
        println!("cargo:rustc-cfg=bytes_original_shared_gate");
        assert!(std::process::Command::new("python3").arg("extract_public.py").status().unwrap().success());
    }
    println!("cargo:rerun-if-changed=../../../src/bytes.rs");
    println!("cargo:rerun-if-changed=../../../src/bytes/bytes_record.rs");
    println!("cargo:rerun-if-changed=../../../src/bytes/vtable_record.rs");
    println!("cargo:rerun-if-changed=../../../src/bytes_mut.rs");
    println!("cargo:rerun-if-changed=extract_public.py");
}
