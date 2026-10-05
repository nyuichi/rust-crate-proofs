use std::{env, fs, path::PathBuf};
use crate::item;

pub fn generate(bytes_mut: &str, generated: &mut String) {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../../..").canonicalize().unwrap();
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut modules = String::new();
    let mut audit = String::new();
    for (module, path, signatures) in [
        ("raw_vec", "src/ownership_proof/raw_vec.rs", vec![
            "pub(crate) unsafe fn borrow_bound<'a>(",
            "pub(crate) unsafe fn borrow_empty_bound<'a>("]),
        ("shared_protocol", "src/ownership_proof/shared_protocol.rs", vec![
            "pub(crate) unsafe fn borrow_packet<'a>("]),
        ("provenance_specs", "src/provenance_specs.rs", vec![
            "pub(crate) fn pointer_addr<T>("]),
    ] {
        println!("cargo:rerun-if-changed={}", root.join(path).display());
        let mut source = fs::read_to_string(root.join(path)).unwrap();
        for signature in signatures {
            let count = source.matches(signature).count();
            assert_eq!(count, if module == "provenance_specs" { 2 } else { 1 });
            source = source.replace(signature,
                &format!("#[cfg_attr(creusot, check(ghost))]\n{signature}"));
            audit.push_str(&format!("{path}: add check(ghost) to {signature}; original body and contracts unchanged\n"));
        }
        // Only rebind the native allocation module location after relocating
        // this otherwise exact source file into OUT_DIR.
        if module == "raw_vec" {
            source = source.replace("#[path = \"../allocation_ops.rs\"]",
                &format!("#[path = {:?}]", root.join("src/allocation_ops.rs")));
        }
        let destination = out.join(format!("readonly_{module}.rs"));
        fs::write(&destination, source).unwrap();
        modules.push_str(&format!("#[path = {:?}]\nmod {module};\n", destination));
    }
    for signature in ["    fn as_slice(&self) -> &[u8] {", "    fn as_ref(&self) -> &[u8] {",
        "    fn kind(&self) -> usize {"] {
        assert_eq!(generated.matches(signature).count(), 1);
        *generated = generated.replace(signature,
            &format!("    #[cfg_attr(creusot, check(ghost))]\n{signature}"));
    }
    *generated = generated.replace("    fn as_ref(&self) -> &[u8] {",
        "    #[cfg_attr(creusot, ensures(result.deep_model() == self.deep_model()))]\n    fn as_ref(&self) -> &[u8] {");
    let deref = item(bytes_mut, "impl Deref for BytesMut {");
    let adapted = deref.replace("    fn deref(&self) -> &[u8] {",
        "    #[cfg_attr(creusot, check(ghost))]\n    #[cfg_attr(creusot, ensures(result.deep_model() == self.deep_model()))]\n    fn deref(&self) -> &[u8] {");
    generated.push_str("use core::ops::Deref;\n");
    generated.push_str(&adapted);
    for marker in ["impl PartialEq<[u8]> for BytesMut {", "impl PartialOrd<[u8]> for BytesMut {",
        "impl PartialEq<BytesMut> for [u8] {", "impl PartialOrd<BytesMut> for [u8] {"] {
        generated.push_str(&crate::comparison_contracts(&item(bytes_mut, marker)));
        generated.push('\n');
    }
    fs::write(out.join("readonly_modules.rs"), modules).unwrap();
    fs::write(out.join("readonly_purity_changes.txt"), audit).unwrap();
}
