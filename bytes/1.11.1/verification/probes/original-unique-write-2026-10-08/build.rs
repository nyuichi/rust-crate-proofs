fn main() {
    println!("cargo:rustc-check-cfg=cfg(bytes_original_unique_gate)");
    println!("cargo:rustc-cfg=bytes_original_unique_gate");
    println!("cargo:rerun-if-changed=../../../src/bytes_mut.rs");
    println!("cargo:rerun-if-changed=extract.py");
    let status = std::process::Command::new("python3").arg("extract.py").status().unwrap();
    assert!(status.success());
}
