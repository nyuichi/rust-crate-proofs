use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const EXPECTED_SOURCE_SHA256: &str =
    "cd4dd691c3ecfedfa7cbf8db095bb1d6eebefbcdc71c67f0304e467a2d45e29d";

struct Fragment<'a> {
    name: &'static str,
    text: &'a str,
    start: usize,
    end: usize,
}

fn sha256(bytes: &[u8]) -> String {
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("sha256sum is required for the extraction receipt");
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "sha256sum failed");
    String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .expect("sha256sum digest")
        .to_owned()
}

fn matching_brace(source: &str, open: usize) -> usize {
    let bytes = source.as_bytes();
    assert_eq!(bytes[open], b'{');
    let mut depth = 0usize;
    let mut i = open;
    let mut state = 0u8; // code, line comment, block comment, string, char
    let mut block_depth = 0usize;
    let mut escaped = false;
    while i < bytes.len() {
        let byte = bytes[i];
        match state {
            0 => match (byte, bytes.get(i + 1).copied()) {
                (b'/', Some(b'/')) => { state = 1; i += 2; continue; }
                (b'/', Some(b'*')) => { state = 2; block_depth = 1; i += 2; continue; }
                (b'"', _) => state = 3,
                (b'\'', _) => {
                    // Do not treat Rust lifetimes such as &'a as char literals.
                    if bytes.get(i + 1) == Some(&b'\\') || bytes.get(i + 2) == Some(&b'\'') {
                        state = 4;
                    }
                }
                (b'{', _) => depth += 1,
                (b'}', _) => {
                    depth -= 1;
                    if depth == 0 { return i + 1; }
                }
                _ => {}
            },
            1 => if byte == b'\n' { state = 0; },
            2 => match (byte, bytes.get(i + 1).copied()) {
                (b'/', Some(b'*')) => { block_depth += 1; i += 2; continue; }
                (b'*', Some(b'/')) => {
                    block_depth -= 1;
                    i += 2;
                    if block_depth == 0 { state = 0; }
                    continue;
                }
                _ => {}
            },
            3 | 4 => {
                if escaped { escaped = false; }
                else if byte == b'\\' { escaped = true; }
                else if (state == 3 && byte == b'"') || (state == 4 && byte == b'\'') { state = 0; }
            }
            _ => unreachable!(),
        }
        i += 1;
    }
    panic!("unterminated source item at byte {open}");
}

fn unique_index(source: &str, marker: &str) -> usize {
    let index = source.find(marker).unwrap_or_else(|| panic!("missing source marker: {marker}"));
    assert!(source[index + marker.len()..].find(marker).is_none(), "duplicate source marker: {marker}");
    index
}

fn item<'a>(source: &'a str, name: &'static str, start_marker: &str, body_marker: &str) -> Fragment<'a> {
    let start = unique_index(source, start_marker);
    let body = unique_index(source, body_marker);
    assert!(start <= body, "item prefix follows its body: {name}");
    let open = body + source[body..].find('{').expect("source item opening brace");
    let end = matching_brace(source, open);
    Fragment { name, text: &source[start..end], start, end }
}

fn lines<'a>(source: &'a str, name: &'static str, marker: &str) -> Fragment<'a> {
    let start = unique_index(source, marker);
    let end = source[start..].find('\n').map(|n| start + n).unwrap_or(source.len());
    Fragment { name, text: &source[start..end], start, end }
}

fn block_lines<'a>(source: &'a str, name: &'static str, marker: &str, count: usize) -> Fragment<'a> {
    let start = unique_index(source, marker);
    let mut end = start;
    for _ in 0..count {
        end = source[end..].find('\n').map(|n| end + n + 1).unwrap_or(source.len());
    }
    Fragment { name, text: &source[start..end], start, end }
}

fn json_quote(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"))
}

fn line_number(source: &str, offset: usize) -> usize {
    source[..offset].bytes().filter(|byte| *byte == b'\n').count() + 1
}

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let crate_root = manifest_dir.join("../../..").canonicalize().expect("bytes crate root");
    let live_source = crate_root.join("src/bytes_mut.rs");
    let snapshot_path = manifest_dir.join("evidence/input-bytes_mut.rs");
    let snapshot_receipt_path = manifest_dir.join("evidence/source-snapshot.json");
    println!("cargo:rerun-if-changed={}", live_source.display());
    println!("cargo:rerun-if-changed={}", snapshot_path.display());
    println!("cargo:rerun-if-changed={}", snapshot_receipt_path.display());
    println!("cargo:rerun-if-changed=build.rs");

    let source_bytes = fs::read(&snapshot_path).expect("read frozen production source snapshot");
    let source_hash = sha256(&source_bytes);
    assert_eq!(source_hash, EXPECTED_SOURCE_SHA256, "frozen source snapshot changed; review and recapture instead of silently regenerating");
    let source = String::from_utf8(source_bytes.clone()).expect("bytes_mut.rs is UTF-8");
    let live_bytes = fs::read(&live_source).expect("read current production source for correspondence");
    let live_hash = sha256(&live_bytes);
    let snapshot_receipt = fs::read(&snapshot_receipt_path).expect("read source snapshot receipt");
    assert!(String::from_utf8_lossy(&snapshot_receipt).contains(EXPECTED_SOURCE_SHA256), "source snapshot receipt does not record the guarded hash");

    let fragments = vec![
        item(&source, "BytesMut", "pub struct BytesMut {", "pub struct BytesMut {"),
        item(&source, "Shared", "struct Shared {", "struct Shared {"),
        lines(&source, "shared-alignment-assertion", "const _: [(); 0 - mem::align_of::<Shared>() % 2] = [];"),
        lines(&source, "KIND_ARC", "const KIND_ARC: usize = 0b0;"),
        lines(&source, "KIND_VEC", "const KIND_VEC: usize = 0b1;"),
        lines(&source, "KIND_MASK", "const KIND_MASK: usize = 0b1;"),
        lines(&source, "MAX_ORIGINAL_CAPACITY_WIDTH", "const MAX_ORIGINAL_CAPACITY_WIDTH: usize = 17;"),
        lines(&source, "MIN_ORIGINAL_CAPACITY_WIDTH", "const MIN_ORIGINAL_CAPACITY_WIDTH: usize = 10;"),
        lines(&source, "ORIGINAL_CAPACITY_MASK", "const ORIGINAL_CAPACITY_MASK: usize = 0b11100;"),
        lines(&source, "ORIGINAL_CAPACITY_OFFSET", "const ORIGINAL_CAPACITY_OFFSET: usize = 2;"),
        lines(&source, "VEC_POS_OFFSET", "const VEC_POS_OFFSET: usize = 5;"),
        lines(&source, "MAX_VEC_POS", "const MAX_VEC_POS: usize = usize::MAX >> VEC_POS_OFFSET;"),
        lines(&source, "NOT_VEC_POS_MASK", "const NOT_VEC_POS_MASK: usize = 0b11111;"),
        block_lines(&source, "PTR_WIDTH_64", "#[cfg(target_pointer_width = \"64\")]\nconst PTR_WIDTH: usize = 64;", 2),
        block_lines(&source, "PTR_WIDTH_32", "#[cfg(target_pointer_width = \"32\")]\nconst PTR_WIDTH: usize = 32;", 2),
        item(&source, "with_capacity", "    #[inline]\n    pub fn with_capacity(capacity: usize) -> BytesMut {", "    pub fn with_capacity(capacity: usize) -> BytesMut {"),
        item(&source, "len", "    #[inline]\n    pub fn len(&self) -> usize {", "    pub fn len(&self) -> usize {"),
        item(&source, "capacity", "    #[inline]\n    pub fn capacity(&self) -> usize {", "    pub fn capacity(&self) -> usize {"),
        item(&source, "from_vec", "    #[inline]\n    pub(crate) fn from_vec(vec: Vec<u8>) -> BytesMut {", "    pub(crate) fn from_vec(vec: Vec<u8>) -> BytesMut {"),
        item(&source, "original_capacity_to_repr", "#[inline]\nfn original_capacity_to_repr(cap: usize) -> usize {", "fn original_capacity_to_repr(cap: usize) -> usize {"),
        item(&source, "vptr", "#[inline]\nfn vptr(ptr: *mut u8) -> NonNull<u8> {", "fn vptr(ptr: *mut u8) -> NonNull<u8> {"),
        item(&source, "invalid_ptr", "#[inline]\nfn invalid_ptr<T>(addr: usize) -> *mut T {", "fn invalid_ptr<T>(addr: usize) -> *mut T {"),
    ];

    // The current native BytesMut representation uses this exact four-field
    // struct and the exact original Shared metadata declaration. No alternate
    // representation or proof-only field is synthesized here.
    let mut generated = String::from("// Generated from the guarded production snapshot.\n");
    for fragment in fragments.iter().filter(|fragment| fragment.name != "with_capacity" && fragment.name != "len" && fragment.name != "capacity" && fragment.name != "from_vec" && fragment.name != "original_capacity_to_repr" && fragment.name != "vptr" && fragment.name != "invalid_ptr") {
        generated.push_str(fragment.text);
        generated.push('\n');
    }
    generated.push_str("\nimpl BytesMut {\n");
    for fragment in fragments.iter().filter(|fragment| matches!(fragment.name, "with_capacity" | "len" | "capacity" | "from_vec")) {
        generated.push_str(fragment.text);
        generated.push('\n');
    }
    generated.push_str("}\n");
    for fragment in fragments.iter().filter(|fragment| matches!(fragment.name, "original_capacity_to_repr" | "vptr" | "invalid_ptr")) {
        generated.push_str(fragment.text);
        generated.push('\n');
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let generated_path = out_dir.join("original_constructor.rs");
    fs::write(&generated_path, &generated).unwrap();
    let evidence_dir = manifest_dir.join("evidence");
    fs::write(evidence_dir.join("generated-original-constructor.rs"), &generated).unwrap();

    let mut receipt = String::from("{\n");
    receipt.push_str(&format!("  \"source_snapshot\": {},\n", json_quote("evidence/input-bytes_mut.rs")));
    receipt.push_str(&format!("  \"source_snapshot_sha256\": {},\n", json_quote(&source_hash)));
    receipt.push_str(&format!("  \"source_snapshot_bytes\": {},\n", source.len()));
    receipt.push_str(&format!("  \"source_snapshot_receipt_sha256\": {},\n", json_quote(&sha256(&snapshot_receipt))));
    receipt.push_str(&format!("  \"production_live_path\": {},\n", json_quote(&live_source.display().to_string())));
    receipt.push_str(&format!("  \"production_live_sha256_at_generation\": {},\n", json_quote(&live_hash)));
    receipt.push_str(&format!("  \"production_matches_snapshot_at_generation\": {},\n", live_hash == source_hash));
    receipt.push_str(&format!("  \"generated_sha256\": {},\n", json_quote(&sha256(generated.as_bytes()))));
    receipt.push_str("  \"fragments\": [\n");
    for (index, fragment) in fragments.iter().enumerate() {
        let comma = if index + 1 == fragments.len() { "" } else { "," };
        receipt.push_str(&format!(
            "    {{\"name\": {}, \"start_byte\": {}, \"end_byte\": {}, \"start_line\": {}, \"end_line\": {}, \"sha256\": {}}}{}\n",
            json_quote(fragment.name), fragment.start, fragment.end,
            line_number(&source, fragment.start), line_number(&source, fragment.end),
            json_quote(&sha256(fragment.text.as_bytes())), comma
        ));
    }
    receipt.push_str("  ]\n}\n");
    fs::write(evidence_dir.join("extraction-manifest.json"), receipt).unwrap();
    println!("cargo:warning=original-constructor extraction: snapshot={source_hash}; live_source_matches={}", live_hash == source_hash);
}
