use std::{env, fs, path::PathBuf, process::Command};

fn extract(source: &str, start_marker: &str, body_marker: &str) -> (String, usize, usize) {
    let start = source.find(start_marker).expect(start_marker);
    assert!(source[start + start_marker.len()..].find(start_marker).is_none(), "duplicate {start_marker}");
    let body_start = source.find(body_marker).expect(body_marker);
    assert!(source[body_start + body_marker.len()..].find(body_marker).is_none(), "duplicate {body_marker}");
    assert!(start <= body_start);
    let open = body_start + source[body_start..].find('{').expect("opening brace");
    let mut depth = 0usize;
    for (offset, byte) in source.as_bytes()[open..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return (source[start..open + offset + 1].to_owned(), start, open + offset + 1);
                }
            }
            _ => {}
        }
    }
    panic!("unterminated item {start_marker}")
}

fn contract(item: String, signature: &str, clause: &str) -> String {
    assert_eq!(item.matches(signature).count(), 1, "unique signature {signature}");
    item.replace(signature, &format!("    #[cfg_attr(creusot, ensures({clause}))]\n{signature}"))
}

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../../..").canonicalize().unwrap();
    let source_path = root.join("src/bytes.rs");
    println!("cargo:rerun-if-changed={}", source_path.display());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=BYTES_PROOF_CALLBACK_STUBS");
    println!("cargo:rerun-if-env-changed=BYTES_OBSERVER_SCOPE");
    println!("cargo:rustc-check-cfg=cfg(bytes_observer_scope, values(\"observers\", \"from_static\", \"all\"))");
    let proof_callback_stubs = env::var_os("BYTES_PROOF_CALLBACK_STUBS").is_some();
    let scope = env::var("BYTES_OBSERVER_SCOPE").unwrap_or_else(|_| "all".to_owned());
    assert!(matches!(scope.as_str(), "observers" | "from_static" | "all"));
    println!("cargo:rustc-cfg=bytes_observer_scope=\"{scope}\"");
    let source = fs::read_to_string(&source_path).expect("read bytes.rs");

    let mut selected = Vec::new();
    for (name, start, body) in [
        ("Bytes", "pub struct Bytes {", "pub struct Bytes {"),
        ("Vtable", "pub(crate) struct Vtable {", "pub(crate) struct Vtable {"),
        ("new", "    #[inline]\n    #[cfg(not(all(loom, test)))]\n    pub const fn new() -> Self {", "    pub const fn new() -> Self {"),
        ("from_static", "    #[inline]\n    #[cfg(not(all(loom, test)))]\n    pub const fn from_static(bytes: &'static [u8]) -> Self {", "    pub const fn from_static(bytes: &'static [u8]) -> Self {"),
        ("len", "    #[inline]\n    pub const fn len(&self) -> usize {", "    pub const fn len(&self) -> usize {"),
        ("is_empty", "    #[inline]\n    pub const fn is_empty(&self) -> bool {", "    pub const fn is_empty(&self) -> bool {"),
        ("STATIC_VTABLE", "const STATIC_VTABLE: Vtable = Vtable {", "const STATIC_VTABLE: Vtable = Vtable {"),
        ("static_clone", "unsafe fn static_clone(", "unsafe fn static_clone("),
        ("static_to_vec", "unsafe fn static_to_vec(", "unsafe fn static_to_vec("),
        ("static_to_mut", "unsafe fn static_to_mut(", "unsafe fn static_to_mut("),
        ("static_is_unique", "fn static_is_unique(", "fn static_is_unique("),
        ("static_drop", "unsafe fn static_drop(", "unsafe fn static_drop("),
    ] {
        let (mut fragment, first, mut last) = extract(&source, start, body);
        if name == "STATIC_VTABLE" {
            let suffix = source[last..].trim_start();
            assert!(suffix.starts_with(';'));
            fragment.push(';');
            last += source[last..].len() - suffix.len() + 1;
        }
        let line = source[..first].bytes().filter(|byte| *byte == b'\n').count() + 1;
        let hash = fnv1a64(fragment.as_bytes());
        selected.push((name, fragment, line, first, last, hash));
    }

    let mut items = selected.iter().map(|(_, text, _, _, _, _)| text.clone()).collect::<Vec<_>>();
    let contracts = [
        ("new", "pub const fn new() -> Self {", "result.len@ == 0"),
        ("from_static", "pub const fn from_static(bytes: &'static [u8]) -> Self {", "result.len@ == bytes@.len()"),
        ("len", "pub const fn len(&self) -> usize {", "result@ == self.len@"),
        ("is_empty", "pub const fn is_empty(&self) -> bool {", "result == (self.len@ == 0)"),
    ];
    for (name, signature, clause) in contracts {
        let index = selected.iter().position(|(found, ..)| *found == name).unwrap();
        items[index] = contract(items[index].clone(), signature, clause);
    }
    if scope == "from_static" && proof_callback_stubs {
        // Retain the exact body but erase const evaluation in this isolated
        // proof view: Creusot cannot materialize the static vtable pointer as
        // a compile-time scalar, while the selected postcondition is runtime
        // field metadata only.
        items[3] = items[3].replacen("pub const fn from_static", "pub fn from_static", 1);
    }

    let mut generated = String::from("#[cfg(creusot)] use creusot_std::prelude::*;\nuse alloc::vec::Vec;\nuse core::{ptr, slice};\nuse core::sync::atomic::AtomicPtr;\nuse crate::BytesMut;\n\n");
    generated.push_str(&items[0]);
    generated.push_str("\n\n");
    generated.push_str(&items[1]);
    generated.push_str("\n\nimpl Bytes {\n");
    if scope == "all" {
        generated.push_str(&items[2]);
        generated.push('\n');
    }
    if scope != "observers" {
        generated.push_str(&items[3]);
        generated.push('\n');
    }
    for index in 4..=5 {
        generated.push_str(&items[index]);
        generated.push('\n');
    }
    generated.push_str("}\n\n");
    if scope != "observers" && proof_callback_stubs {
        // Function-item-to-function-pointer casts in the exact table exceed
        // the pinned frontend. Keep the actual table source in snapshots and
        // native builds; this immutable uninterpreted static contributes no
        // callback, value, or protocol fact to the metadata-only proof.
        generated.push_str("unsafe extern \"Rust\" { safe static STATIC_VTABLE: Vtable; }\n\n");
    } else if scope != "observers" {
        generated.push_str(&items[6]);
        generated.push_str("\n\n");
        for index in 7..=11 {
            generated.push_str(&items[index]);
            generated.push('\n');
        }
    }

    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("actual_bytes_observers.rs"), generated).unwrap();
    fs::write(out.join("bytes_source_snapshot.rs"), source).unwrap();
    if scope != "observers" {
        let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let sysroot = Command::new(&rustc).args(["--print", "sysroot"]).output().expect("run rustc --print sysroot");
        assert!(sysroot.status.success(), "rustc --print sysroot failed");
        let sysroot = String::from_utf8(sysroot.stdout).expect("sysroot utf8").trim().to_owned();
        let version = Command::new(&rustc).args(["--version", "--verbose"]).output().expect("run rustc --version");
        assert!(version.status.success(), "rustc --version failed");
        let version = String::from_utf8(version.stdout).expect("rustc version utf8");
        let atomic_source_path = PathBuf::from(&sysroot).join("lib/rustlib/src/rust/library/core/src/sync/atomic.rs");
        println!("cargo:rerun-if-changed={}", atomic_source_path.display());
        let atomic_source = fs::read_to_string(&atomic_source_path).expect("read pinned core AtomicPtr source");
        let marker = "    pub const fn new(p: *mut T) -> AtomicPtr<T> {";
        let (definition, start, _) = extract(&atomic_source, marker, marker);
        let line = atomic_source[..start].bytes().filter(|byte| *byte == b'\n').count() + 1;
        fs::write(out.join("std_atomic_ptr_new_source.txt"), &definition).unwrap();
        fs::write(out.join("std_atomic_ptr_new_source_metadata.txt"), format!(
            "Generic standard-library constructor trusted boundary used by the source-sliced proof.\n\
             rustc sysroot: {sysroot}\n\
             rustc version:\n{version}\
             definition: {}/library/core/src/sync/atomic.rs:{line}\n\
             full atomic.rs FNV-1a-64: {:016x}\n\
             extracted definition FNV-1a-64: {:016x}\n\
             Boundary contract: ensures(true) only; no stored-value relation, pointer authority, permission, memory-ordering fact, refcount, or bytes ownership/protocol fact.\n",
            sysroot,
            fnv1a64(atomic_source.as_bytes()),
            fnv1a64(definition.as_bytes()),
        )).unwrap();
    }
    let mut fragments = String::new();
    for (name, text, line, first, last, hash) in &selected {
        fragments.push_str(&format!("{name}: src/bytes.rs:{line}, bytes {first}..{last}, fnv1a64={hash:016x}, {} bytes\n", text.len()));
    }
    fragments.push_str(&format!("Selected generated scope: {scope}. Exact STATIC_VTABLE and callback source are retained byte-for-byte in the source snapshot and native build. When BYTES_PROOF_CALLBACK_STUBS is set outside the all scope, proof scope substitutes an immutable uninterpreted external STATIC_VTABLE because the pinned frontend rejects callback function-item-to-function-pointer casts; no table value, callback, or ownership/refcount/protocol fact is supplied. In the from_static-only scope, the method body is exact but its const qualifier is omitted in the proof view because Creusot cannot materialize a static-vtable pointer as a compile-time scalar. The all scope retains exact const qualifiers. Only field metadata postconditions are in scope.\n"));
    fs::write(out.join("source_fragments.txt"), fragments).unwrap();
    let native_callbacks = selected.iter().skip(7).map(|(_, text, ..)| text.as_str()).collect::<Vec<_>>().join("\n\n");
    fs::write(out.join("static_callbacks_source_snapshot.rs"), native_callbacks).unwrap();
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
