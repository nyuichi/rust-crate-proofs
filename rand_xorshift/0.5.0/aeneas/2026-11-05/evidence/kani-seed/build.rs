use std::{env,fs,path::PathBuf};
fn main() {
 let source=PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../src/lib.rs");
 println!("cargo:rerun-if-changed={}",source.display());
 let bytes=fs::read(&source).unwrap();
 let observer=br#"
/// Verification-only state observer; invokes the unchanged original seed constructor.
pub fn verification_from_seed(seed: [u8;16]) -> [u32;4] {
 let rng = <XorShiftRng as SeedableRng>::from_seed(seed);
 [rng.x.0,rng.y.0,rng.z.0,rng.w.0]
}
/// Verification-only observer of one unchanged original next-word transition.
pub fn verification_next_state(words: [u32;4]) -> [u32;4] {
 let mut rng = XorShiftRng { x:w(words[0]),y:w(words[1]),z:w(words[2]),w:w(words[3]) };
 let _ = rng.try_next_u32();
 [rng.x.0,rng.y.0,rng.z.0,rng.w.0]
}
"#;
 let mut generated=bytes.clone();generated.extend_from_slice(observer);
 assert_eq!(&generated[..bytes.len()],bytes.as_slice());
 let out=PathBuf::from(env::var("OUT_DIR").unwrap());
 let file=out.join("original.rs");fs::write(&file,generated).unwrap();
 fs::write(out.join("modules.rs"),format!("#[path = {:?}] mod original;\n",file)).unwrap();
}
