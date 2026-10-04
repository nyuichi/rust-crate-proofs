use std::{env, fs, path::PathBuf};

struct Fragment<'a> {
    name: &'static str,
    text: &'a str,
    start: usize,
    end: usize,
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
                (b'/', Some(b'/')) => {
                    state = 1;
                    i += 2;
                    continue;
                }
                (b'/', Some(b'*')) => {
                    state = 2;
                    block_depth = 1;
                    i += 2;
                    continue;
                }
                (b'"', _) => state = 3,
                (b'\'', _) => state = 4,
                (b'{', _) => depth += 1,
                (b'}', _) => {
                    depth -= 1;
                    if depth == 0 {
                        return i + 1;
                    }
                }
                _ => {}
            },
            1 => {
                if byte == b'\n' {
                    state = 0;
                }
            }
            2 => match (byte, bytes.get(i + 1).copied()) {
                (b'/', Some(b'*')) => {
                    block_depth += 1;
                    i += 2;
                    continue;
                }
                (b'*', Some(b'/')) => {
                    block_depth -= 1;
                    i += 2;
                    if block_depth == 0 {
                        state = 0;
                    }
                    continue;
                }
                _ => {}
            },
            3 | 4 => {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if (state == 3 && byte == b'"') || (state == 4 && byte == b'\'') {
                    state = 0;
                }
            }
            _ => unreachable!(),
        }
        i += 1;
    }
    panic!("unterminated Rust item starting at byte {open}");
}

fn unique_index(source: &str, marker: &str) -> usize {
    let first = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker: {marker}"));
    assert_eq!(
        source[first + marker.len()..].find(marker),
        None,
        "duplicate marker: {marker}"
    );
    first
}

fn extract_item<'a>(
    source: &'a str,
    name: &'static str,
    start_marker: &str,
    body_marker: &str,
) -> Fragment<'a> {
    let start = unique_index(source, start_marker);
    let body = unique_index(source, body_marker);
    assert!(start <= body, "item prefix follows its body marker: {name}");
    let open = body + source[body..].find('{').expect("item body opening brace");
    let end = matching_brace(source, open);
    Fragment {
        name,
        text: &source[start..end],
        start,
        end,
    }
}

fn extract_line<'a>(source: &'a str, name: &'static str, marker: &str) -> Fragment<'a> {
    let start = unique_index(source, marker);
    let end = start + source[start..].find('\n').unwrap_or(source.len() - start);
    Fragment {
        name,
        text: &source[start..end],
        start,
        end,
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let source_path = manifest_dir.join("../../../src/bytes_mut.rs");
    println!("cargo:rerun-if-changed={}", source_path.display());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-cfg=bytes_proof_probe");
    println!("cargo:rustc-check-cfg=cfg(bytes_proof_probe)");
    let source = fs::read_to_string(&source_path).unwrap();
    let shared = extract_item(&source, "Shared", "struct Shared {", "struct Shared {");
    let buffer = extract_item(&source, "SharedBuffer", "struct SharedBuffer {", "struct SharedBuffer {");
    let drop_buffer = extract_item(&source, "SharedBuffer::drop", "impl Drop for SharedBuffer {", "impl Drop for SharedBuffer {");
    let start = unique_index(&source, "// BEGIN EXACT SEQUENTIAL SHARED CONTROL GATE");
    let end = unique_index(&source, "// END EXACT SEQUENTIAL SHARED CONTROL GATE");
    let control = Fragment {name: "sequential_shared_control", text: &source[start..end], start, end};
    let mut generated = String::from("// Exact source fragments; build.rs records offsets and hashes.\nuse alloc::{vec::Vec, boxed::Box};\nuse core::ptr::NonNull;\nuse core::sync::atomic::AtomicUsize;\nuse creusot_std::prelude::*;\n");
    for f in [&shared, &buffer, &control] { generated.push_str(f.text); generated.push_str("\n"); }
    // Creusot does not model implicit Drop. Keep the exact Drop body in native
    // smoke tests, where emptying the buffer must prevent a second A free.
    generated.push_str("#[cfg(not(creusot))]\n");
    generated.push_str(drop_buffer.text);
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("actual_shared.rs"), generated).unwrap();
    let mut records = String::new();
    for f in [&shared, &buffer, &control, &drop_buffer] {
        records.push_str(&format!("{} {} {} {:016x}\n",f.name,f.start,f.end,fnv1a64(f.text.as_bytes())));
    }
    records.push_str("adaptation: cfg(bytes_proof_probe) selects same-word field adapters; exact SharedBuffer::drop is native-only because automatic Drop is not modeled\n");
    fs::write(out.join("source_fragments.txt"), records).unwrap();
}
