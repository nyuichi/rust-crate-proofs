use std::{env, fs, path::PathBuf};

fn extract(source: &str, marker: &str) -> (String, usize, usize) {
    let start = source.find(marker).expect(marker);
    assert!(source[start + marker.len()..].find(marker).is_none(), "duplicate {marker}");
    let open = start + source[start..].find('{').expect("opening brace");
    let mut depth = 0usize;
    for (offset, byte) in source.as_bytes()[open..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    let mut end = open + offset + 1;
                    if marker.starts_with("const ") || marker.starts_with("static ") {
                        let suffix = source[end..].trim_start();
                        assert!(suffix.starts_with(';'));
                        end += source[end..].len() - suffix.len() + 1;
                    }
                    return (source[start..end].to_owned(), start, end);
                }
            }
            _ => {}
        }
    }
    panic!("unterminated item {marker}")
}

fn signature(source: &str, marker: &str) -> String {
    let start = source.find(marker).expect(marker);
    let open = start + source[start..].find('{').expect("function body");
    format!("{};", source[start..open].trim_end())
}

fn attach_contract(item: String, signature: &str, clauses: &[(&str, &str)]) -> String {
    assert_eq!(item.matches(signature).count(), 1, "unique signature {signature}");
    let mut attributes = String::new();
    for (kind, clause) in clauses {
        attributes.push_str(&format!("    #[cfg_attr(creusot, {kind}({clause}))]\n"));
    }
    item.replace(signature, &format!("{attributes}{signature}"))
}

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../../..").canonicalize().unwrap();
    let source_path = root.join("src/bytes.rs");
    let bytes_mut_source_path = root.join("src/bytes_mut.rs");
    println!("cargo:rerun-if-changed={}", source_path.display());
    println!("cargo:rerun-if-changed={}", bytes_mut_source_path.display());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=BYTES_TRUNCATE_PROOF_STUBS");
    let proof_stubs = env::var_os("BYTES_TRUNCATE_PROOF_STUBS").is_some();
    let source = fs::read_to_string(&source_path).expect("read bytes.rs");
    let bytes_mut_source = fs::read_to_string(&bytes_mut_source_path).expect("read bytes_mut.rs");
    let (bytes_mut_shared_vtable, bytes_mut_shared_start, bytes_mut_shared_end) =
        extract(&bytes_mut_source, "static SHARED_VTABLE: Vtable = Vtable {");
    let bytes_mut_shared_line = bytes_mut_source[..bytes_mut_shared_start].bytes().filter(|byte| *byte == b'\n').count() + 1;

    let markers = [
        ("Bytes", "pub struct Bytes {"),
        ("Vtable", "pub(crate) struct Vtable {"),
        ("from_static", "    pub const fn from_static(bytes: &'static [u8]) -> Self {"),
        ("len", "    pub const fn len(&self) -> usize {"),
        ("truncate", "    pub fn truncate(&mut self, len: usize) {"),
        ("clear", "    pub fn clear(&mut self) {"),
        ("STATIC_VTABLE", "const STATIC_VTABLE: Vtable = Vtable {"),
        ("OWNED_VTABLE", "const VTABLE: Vtable = Vtable {"),
        ("SHARED_VTABLE_BYTES", "static SHARED_VTABLE: Vtable = Vtable {"),
        ("PROMOTABLE_EVEN_VTABLE", "static PROMOTABLE_EVEN_VTABLE: Vtable = Vtable {"),
        ("PROMOTABLE_ODD_VTABLE", "static PROMOTABLE_ODD_VTABLE: Vtable = Vtable {"),
        ("static_clone", "unsafe fn static_clone("),
        ("static_to_vec", "unsafe fn static_to_vec("),
        ("static_to_mut", "unsafe fn static_to_mut("),
        ("static_is_unique", "fn static_is_unique("),
        ("static_drop", "unsafe fn static_drop("),
        ("promotable_even_clone", "unsafe fn promotable_even_clone("),
        ("promotable_even_to_vec", "unsafe fn promotable_even_to_vec("),
        ("promotable_even_to_mut", "unsafe fn promotable_even_to_mut("),
        ("promotable_even_drop", "unsafe fn promotable_even_drop("),
        ("promotable_is_unique", "fn promotable_is_unique("),
        ("promotable_odd_clone", "unsafe fn promotable_odd_clone("),
        ("promotable_odd_to_vec", "unsafe fn promotable_odd_to_vec("),
        ("promotable_odd_to_mut", "unsafe fn promotable_odd_to_mut("),
        ("promotable_odd_drop", "unsafe fn promotable_odd_drop("),
    ];
    let mut selected = Vec::new();
    for (name, marker) in markers {
        let (text, start, end) = extract(&source, marker);
        let line = source[..start].bytes().filter(|byte| *byte == b'\n').count() + 1;
        let hash = fnv1a64(text.as_bytes());
        selected.push((name, marker, text, line, start, end, hash));
    }
    let find = |name: &str| selected.iter().find(|entry| entry.0 == name).unwrap().2.clone();

    let nonpromotable = "!self.vtable.promotable";
    let mut from_static = find("from_static");
    from_static = attach_contract(from_static, "pub const fn from_static(bytes: &'static [u8]) -> Self {", &[("ensures", "result.len@ == bytes@.len()")]);
    let mut len = find("len");
    len = attach_contract(len, "pub const fn len(&self) -> usize {", &[("ensures", "result@ == self.len@")]);
    let mut truncate = find("truncate");
    truncate = attach_contract(truncate, "pub fn truncate(&mut self, len: usize) {", &[
        ("requires", nonpromotable),
        ("ensures", "(^self).len@ == (if len@ < self.len@ { len@ } else { self.len@ })"),
    ]);
    let mut clear = find("clear");
    clear = attach_contract(clear, "pub fn clear(&mut self) {", &[
        ("requires", nonpromotable),
        ("ensures", "(^self).len@ == 0"),
    ]);

    let mut generated = String::from("#[cfg(creusot)] use creusot_std::prelude::*;\nuse alloc::vec::Vec;\nuse core::{ptr, slice};\nuse core::sync::atomic::AtomicPtr;\nuse crate::BytesMut;\n\n");
    generated.push_str(&find("Bytes"));
    generated.push_str("\n\n");
    generated.push_str(&find("Vtable"));
    generated.push_str("\n\nimpl Bytes {\n");
    if !proof_stubs {
        generated.push_str(&from_static);
        generated.push('\n');
    }
    for item in [&len, &truncate, &clear] {
        generated.push_str(item);
        generated.push('\n');
    }
    generated.push_str("    #[cfg_attr(creusot, requires(false))]\n    fn split_off(&mut self, _at: usize) -> Self { panic!(\"unreachable promotable branch in this conditional subset\") }\n}\n\n");
    if proof_stubs {
        generated.push_str("#[cfg(creusot)]\n#[requires(!bytes.vtable.promotable)]\n#[ensures(result@ == (if requested@ < bytes.len@ { requested@ } else { bytes.len@ }))]\npub fn truncate_metadata_caller(mut bytes: Bytes, requested: usize) -> usize {\n    bytes.truncate(requested);\n    let len = bytes.len();\n    core::mem::forget(bytes);\n    len\n}\n\n");
        generated.push_str("#[cfg(creusot)]\n#[requires(!bytes.vtable.promotable)]\n#[ensures(result@ == 0)]\npub fn clear_metadata_caller(mut bytes: Bytes) -> usize {\n    bytes.clear();\n    let len = bytes.len();\n    core::mem::forget(bytes);\n    len\n}\n\n");
    } else {
        generated.push_str(&find("STATIC_VTABLE"));
        generated.push_str("\n\n");
        generated.push_str(&find("PROMOTABLE_EVEN_VTABLE"));
        generated.push_str("\n\n");
        generated.push_str(&find("PROMOTABLE_ODD_VTABLE"));
        generated.push_str("\n\n");
    }

    let callbacks = [
        "static_clone", "static_to_vec", "static_to_mut", "static_is_unique", "static_drop",
        "promotable_even_clone", "promotable_even_to_vec", "promotable_even_to_mut", "promotable_even_drop",
        "promotable_is_unique", "promotable_odd_clone", "promotable_odd_to_vec", "promotable_odd_to_mut", "promotable_odd_drop",
    ];
    if proof_stubs {
        for callback in callbacks {
            generated.push_str("unsafe extern \"Rust\" {\n    ");
            generated.push_str(&signature(&source, selected.iter().find(|entry| entry.0 == callback).unwrap().1));
            generated.push_str("\n}\n");
        }
    } else {
        for callback in callbacks {
            if callback.starts_with("static_") {
                generated.push_str(&find(callback));
                generated.push('\n');
            } else {
                let sig = signature(&source, selected.iter().find(|entry| entry.0 == callback).unwrap().1);
                let sig = sig.strip_suffix(';').unwrap();
                generated.push_str(sig);
                generated.push_str(" { panic!(\"unselected promotable callback stand-in\") }\n");
            }
        }
    }

    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("actual_bytes_truncate.rs"), generated).unwrap();
    fs::write(out.join("bytes_source_snapshot.rs"), &source).unwrap();
    fs::write(out.join("bytes_mut_source_snapshot.rs"), &bytes_mut_source).unwrap();
    fs::write(out.join("bytes_mut_shared_vtable_fragment.rs"), &bytes_mut_shared_vtable).unwrap();
    let mut manifest = String::new();
    for (name, _marker, text, line, start, end, hash) in &selected {
        manifest.push_str(&format!("{name}: src/bytes.rs:{line}, bytes {start}..{end}, fnv1a64={hash:016x}, {} bytes\n", text.len()));
    }
    manifest.push_str(&format!("bytes_mut SHARED_VTABLE: src/bytes_mut.rs:{bytes_mut_shared_line}, bytes {bytes_mut_shared_start}..{bytes_mut_shared_end}, fnv1a64={:016x}, {} bytes\n", fnv1a64(bytes_mut_shared_vtable.as_bytes()), bytes_mut_shared_vtable.len()));
    manifest.push_str(&format!("Proof callback declarations substituted for bodies: {}. Native source includes exact STATIC_VTABLE and both PROMOTABLE table initializers; proof scope omits all three because the metadata methods need only the exact Vtable.promotable field and explicit !self.vtable.promotable precondition. The proof-only from_static method is omitted to avoid static pointer construction. The split_off panic adapter has requires(false), and the false branch is established from the tag precondition.\n", if proof_stubs { "yes" } else { "no" }));
    fs::write(out.join("source_fragments.txt"), manifest).unwrap();
    let callbacks_source = selected.iter()
        .filter(|entry| matches!(entry.0,
            "static_clone" | "static_to_vec" | "static_to_mut" | "static_is_unique" | "static_drop" |
            "promotable_even_clone" | "promotable_even_to_vec" | "promotable_even_to_mut" | "promotable_even_drop" | "promotable_is_unique" |
            "promotable_odd_clone" | "promotable_odd_to_vec" | "promotable_odd_to_mut" | "promotable_odd_drop"))
        .map(|entry| entry.2.as_str()).collect::<Vec<_>>().join("\n\n");
    fs::write(out.join("callback_source_snapshot.rs"), callbacks_source).unwrap();
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
