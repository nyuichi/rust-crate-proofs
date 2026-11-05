use std::{env, fs, path::PathBuf};
fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let source = root.join("../../src/lib.rs");
    println!("cargo:rerun-if-changed={}", source.display());
    let bytes = fs::read(&source).unwrap();
    let probe = br#"
/// Verification observer of the original next_u32 body and complete state.
pub fn verification_probe(words: [u32; 4]) -> (u32, [u32; 4]) {
    let mut state = XorShiftRng { x: w(words[0]), y: w(words[1]),
        z: w(words[2]), w: w(words[3]) };
    let output = state.try_next_u32().unwrap();
    (output, [state.x.0, state.y.0, state.z.0, state.w.0])
}
"#;
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut generated = bytes.clone();
    generated.extend_from_slice(probe);
    assert_eq!(&generated[..bytes.len()], &bytes[..]);
    let module = out.join("original.rs");
    fs::write(&module, generated).unwrap();
    fs::write(out.join("modules.rs"), format!("#[path = {:?}] mod original;\n", module)).unwrap();
}
