fn main(){
 for cfg in ["bytes_original_unique_gate","bytes_original_freeze_gate"] {
  println!("cargo:rustc-check-cfg=cfg({cfg})");println!("cargo:rustc-cfg={cfg}");
 }
 for path in ["../../../src/bytes_mut.rs","../../../src/bytes.rs","extract.py"] {println!("cargo:rerun-if-changed={path}");}
 assert!(std::process::Command::new("python3").arg("extract.py").status().unwrap().success());
}
