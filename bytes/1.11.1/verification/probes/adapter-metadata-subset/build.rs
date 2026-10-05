use std::{env, fs, path::PathBuf};

fn extract_item(source: &str, item_start: &str, body_start: &str) -> (String, usize, usize) {
    let start = source
        .find(item_start)
        .unwrap_or_else(|| panic!("missing {item_start}"));
    assert!(
        source[start + item_start.len()..]
            .find(item_start)
            .is_none(),
        "duplicate {item_start}"
    );
    let body = source
        .find(body_start)
        .unwrap_or_else(|| panic!("missing {body_start}"));
    assert!(
        source[body + body_start.len()..].find(body_start).is_none(),
        "duplicate {body_start}"
    );
    assert!(start <= body, "item marker follows body marker");
    let open = body + source[body..].find('{').expect("opening brace");
    let mut depth = 0usize;
    for (offset, byte) in source.as_bytes()[open..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return (
                        source[start..open + offset + 1].to_owned(),
                        start,
                        open + offset + 1,
                    );
                }
            }
            _ => {}
        }
    }
    panic!("unterminated item {item_start}")
}

fn method(source: &str, signature: &str) -> (String, usize, usize) {
    extract_item(source, signature, signature)
}

fn with_contract(mut item: String, signature: &str, clauses: &[&str]) -> String {
    assert_eq!(
        item.matches(signature).count(),
        1,
        "unique signature {signature}"
    );
    let attributes = clauses
        .iter()
        .map(|clause| format!("    #[cfg_attr(creusot, ensures({clause}))]\n"))
        .collect::<String>();
    item = item.replace(signature, &format!("{attributes}{signature}"));
    item
}

fn rewrite_once(mut item: String, before: &str, after: &str) -> String {
    assert_eq!(
        item.matches(before).count(),
        1,
        "unique contract fragment {before}"
    );
    item = item.replace(before, after);
    item
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
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let crate_root = manifest.join("../../..").canonicalize().unwrap();
    let source_root = crate_root.join("src/buf");
    let take_path = source_root.join("take.rs");
    let limit_path = source_root.join("limit.rs");
    let chain_path = source_root.join("chain.rs");
    let reader_path = source_root.join("reader.rs");
    let writer_path = source_root.join("writer.rs");
    for path in [
        &take_path,
        &limit_path,
        &chain_path,
        &reader_path,
        &writer_path,
    ] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rerun-if-changed=build.rs");

    let take = fs::read_to_string(&take_path).expect("read take.rs");
    let limit = fs::read_to_string(&limit_path).expect("read limit.rs");
    let chain = fs::read_to_string(&chain_path).expect("read chain.rs");
    let reader = fs::read_to_string(&reader_path).expect("read reader.rs");
    let writer = fs::read_to_string(&writer_path).expect("read writer.rs");

    let mut selected = Vec::new();
    let take_struct = extract_item(&take, "pub struct Take<T> {", "pub struct Take<T> {");
    selected.push(("Take", "take.rs", take_struct.clone()));
    let take_new = extract_item(&take,
        "#[cfg_attr(creusot, ensures(result.inner == inner))]\n#[cfg_attr(creusot, ensures(result.limit == limit))]\npub fn new<T>(inner: T, limit: usize) -> Take<T> {",
        "pub fn new<T>(inner: T, limit: usize) -> Take<T> {");
    selected.push(("Take::new", "take.rs", take_new.clone()));
    let take_into = method(&take, "    pub fn into_inner(self) -> T {");
    selected.push(("Take::into_inner", "take.rs", take_into.clone()));
    let take_ref = extract_item(&take,
        "    #[cfg_attr(creusot, check(ghost))]\n    #[cfg_attr(creusot, ensures(*result == self.inner))]\n    pub fn get_ref(&self) -> &T {",
        "    pub fn get_ref(&self) -> &T {");
    selected.push(("Take::get_ref", "take.rs", take_ref.clone()));
    let take_mut = method(&take, "    pub fn get_mut(&mut self) -> &mut T {");
    selected.push(("Take::get_mut", "take.rs", take_mut.clone()));
    let take_limit = extract_item(&take,
        "    #[cfg_attr(creusot, check(ghost))]\n    #[cfg_attr(creusot, ensures(result == self.limit))]\n    pub fn limit(&self) -> usize {",
        "    pub fn limit(&self) -> usize {");
    selected.push(("Take::limit", "take.rs", take_limit.clone()));
    let take_set = method(&take, "    pub fn set_limit(&mut self, lim: usize) {");
    selected.push(("Take::set_limit", "take.rs", take_set.clone()));

    let limit_struct = extract_item(&limit, "pub struct Limit<T> {", "pub struct Limit<T> {");
    selected.push(("Limit", "limit.rs", limit_struct.clone()));
    let limit_new = method(
        &limit,
        "pub(super) fn new<T>(inner: T, limit: usize) -> Limit<T> {",
    );
    selected.push(("Limit::new", "limit.rs", limit_new.clone()));
    let limit_into = method(&limit, "    pub fn into_inner(self) -> T {");
    selected.push(("Limit::into_inner", "limit.rs", limit_into.clone()));
    let limit_ref = method(&limit, "    pub fn get_ref(&self) -> &T {");
    selected.push(("Limit::get_ref", "limit.rs", limit_ref.clone()));
    let limit_mut = method(&limit, "    pub fn get_mut(&mut self) -> &mut T {");
    selected.push(("Limit::get_mut", "limit.rs", limit_mut.clone()));
    let limit_limit = method(&limit, "    pub fn limit(&self) -> usize {");
    selected.push(("Limit::limit", "limit.rs", limit_limit.clone()));
    let limit_set = method(&limit, "    pub fn set_limit(&mut self, lim: usize) {");
    selected.push(("Limit::set_limit", "limit.rs", limit_set.clone()));

    let chain_struct = extract_item(
        &chain,
        "pub struct Chain<T, U> {",
        "pub struct Chain<T, U> {",
    );
    selected.push(("Chain", "chain.rs", chain_struct.clone()));
    let chain_new = extract_item(&chain,
        "    #[cfg_attr(creusot, ensures(result.a == a))]\n    #[cfg_attr(creusot, ensures(result.b == b))]\n    pub(crate) fn new(a: T, b: U) -> Chain<T, U> {",
        "    pub(crate) fn new(a: T, b: U) -> Chain<T, U> {");
    selected.push(("Chain::new", "chain.rs", chain_new.clone()));
    let chain_into = method(&chain, "    pub fn into_inner(self) -> (T, U) {");
    selected.push(("Chain::into_inner", "chain.rs", chain_into.clone()));

    let reader_struct = extract_item(&reader, "pub struct Reader<B> {", "pub struct Reader<B> {");
    selected.push(("Reader", "reader.rs", reader_struct.clone()));
    let reader_new = extract_item(
        &reader,
        "#[cfg_attr(creusot, ensures(result.buf == buf))]\npub fn new<B>(buf: B) -> Reader<B> {",
        "pub fn new<B>(buf: B) -> Reader<B> {",
    );
    selected.push(("Reader::new", "reader.rs", reader_new.clone()));
    let reader_ref = extract_item(&reader,
        "    #[cfg_attr(creusot, check(ghost))]\n    #[cfg_attr(creusot, ensures(*result == self.buf))]\n    pub fn get_ref(&self) -> &B {",
        "    pub fn get_ref(&self) -> &B {");
    selected.push(("Reader::get_ref", "reader.rs", reader_ref.clone()));
    let reader_mut = method(&reader, "    pub fn get_mut(&mut self) -> &mut B {");
    selected.push(("Reader::get_mut", "reader.rs", reader_mut.clone()));
    let reader_into = method(&reader, "    pub fn into_inner(self) -> B {");
    selected.push(("Reader::into_inner", "reader.rs", reader_into.clone()));

    let writer_struct = extract_item(&writer, "pub struct Writer<B> {", "pub struct Writer<B> {");
    selected.push(("Writer", "writer.rs", writer_struct.clone()));
    let writer_new = extract_item(
        &writer,
        "#[cfg_attr(creusot, ensures(result.buf == buf))]\npub fn new<B>(buf: B) -> Writer<B> {",
        "pub fn new<B>(buf: B) -> Writer<B> {",
    );
    selected.push(("Writer::new", "writer.rs", writer_new.clone()));
    let writer_ref = extract_item(&writer,
        "    #[cfg_attr(creusot, check(ghost))]\n    #[cfg_attr(creusot, ensures(*result == self.buf))]\n    pub fn get_ref(&self) -> &B {",
        "    pub fn get_ref(&self) -> &B {");
    selected.push(("Writer::get_ref", "writer.rs", writer_ref.clone()));
    let writer_mut = method(&writer, "    pub fn get_mut(&mut self) -> &mut B {");
    selected.push(("Writer::get_mut", "writer.rs", writer_mut.clone()));
    let writer_into = method(&writer, "    pub fn into_inner(self) -> B {");
    selected.push(("Writer::into_inner", "writer.rs", writer_into.clone()));

    let mut items = Vec::new();
    for (name, _, fragment) in &selected {
        let item = match *name {
            "Take::new" => rewrite_once(
                rewrite_once(
                    fragment.0.clone(),
                    "result.inner == inner",
                    "result.inner_logic() == inner",
                ),
                "result.limit == limit",
                "result.limit_logic() == limit",
            ),
            "Take::into_inner" => with_contract(
                fragment.0.clone(),
                "    pub fn into_inner(self) -> T {",
                &["result == self.inner_logic()"],
            ),
            "Take::get_ref" => rewrite_once(
                fragment.0.clone(),
                "*result == self.inner",
                "*result == self.inner_logic()",
            ),
            "Take::get_mut" => with_contract(
                fragment.0.clone(),
                "    pub fn get_mut(&mut self) -> &mut T {",
                &[
                    "^result == (^self).inner_logic()",
                    "(^self).limit_logic() == self.limit_logic()",
                ],
            ),
            "Take::limit" => rewrite_once(
                fragment.0.clone(),
                "result == self.limit",
                "result == self.limit_logic()",
            ),
            "Take::set_limit" => with_contract(
                fragment.0.clone(),
                "    pub fn set_limit(&mut self, lim: usize) {",
                &[
                    "(^self).limit_logic() == lim",
                    "(^self).inner_logic() == self.inner_logic()",
                ],
            ),
            "Limit::new" => with_contract(
                fragment.0.clone(),
                "pub(super) fn new<T>(inner: T, limit: usize) -> Limit<T> {",
                &[
                    "result.inner_logic() == inner",
                    "result.limit_logic() == limit",
                ],
            ),
            "Limit::into_inner" => with_contract(
                fragment.0.clone(),
                "    pub fn into_inner(self) -> T {",
                &["result == self.inner_logic()"],
            ),
            "Limit::get_ref" => with_contract(
                fragment.0.clone(),
                "    pub fn get_ref(&self) -> &T {",
                &["*result == self.inner_logic()"],
            ),
            "Limit::get_mut" => with_contract(
                fragment.0.clone(),
                "    pub fn get_mut(&mut self) -> &mut T {",
                &[
                    "^result == (^self).inner_logic()",
                    "(^self).limit_logic() == self.limit_logic()",
                ],
            ),
            "Limit::limit" => with_contract(
                fragment.0.clone(),
                "    pub fn limit(&self) -> usize {",
                &["result == self.limit_logic()"],
            ),
            "Limit::set_limit" => with_contract(
                fragment.0.clone(),
                "    pub fn set_limit(&mut self, lim: usize) {",
                &[
                    "(^self).limit_logic() == lim",
                    "(^self).inner_logic() == self.inner_logic()",
                ],
            ),
            "Chain::new" => rewrite_once(
                rewrite_once(fragment.0.clone(), "result.a == a", "result.a_logic() == a"),
                "result.b == b",
                "result.b_logic() == b",
            ),
            "Chain::into_inner" => with_contract(
                fragment.0.clone(),
                "    pub fn into_inner(self) -> (T, U) {",
                &["result.0 == self.a_logic()", "result.1 == self.b_logic()"],
            ),
            "Reader::new" | "Writer::new" => rewrite_once(
                fragment.0.clone(),
                "result.buf == buf",
                "result.buf_logic() == buf",
            ),
            "Reader::get_ref" | "Writer::get_ref" => rewrite_once(
                fragment.0.clone(),
                "*result == self.buf",
                "*result == self.buf_logic()",
            ),
            "Reader::get_mut" | "Writer::get_mut" => with_contract(
                fragment.0.clone(),
                "    pub fn get_mut(&mut self) -> &mut B {",
                &["^result == (^self).buf_logic()"],
            ),
            "Reader::into_inner" | "Writer::into_inner" => with_contract(
                fragment.0.clone(),
                "    pub fn into_inner(self) -> B {",
                &["result == self.buf_logic()"],
            ),
            _ => fragment.0.clone(),
        };
        items.push((*name, item));
    }
    let get = |name: &str| -> &str {
        items
            .iter()
            .find(|(found, _)| *found == name)
            .unwrap()
            .1
            .as_str()
    };

    let mut generated = String::from("#[allow(dead_code)]\n\n");
    generated.push_str("pub mod take {\n#[cfg(creusot)]\nuse creusot_std::prelude::*;\n");
    generated.push_str(get("Take"));
    generated.push_str("\n");
    generated.push_str(get("Take::new"));
    generated.push_str("\nimpl<T> Take<T> {\n");
    generated.push_str("#[cfg(creusot)]\n#[logic(open(self))]\npub(crate) fn inner_logic(self) -> T { self.inner }\n#[cfg(creusot)]\n#[logic(open(self))]\npub(crate) fn limit_logic(self) -> usize { self.limit }\n");
    for name in [
        "Take::into_inner",
        "Take::get_ref",
        "Take::get_mut",
        "Take::limit",
        "Take::set_limit",
    ] {
        generated.push_str(get(name));
        generated.push('\n');
    }
    generated.push_str("}\n}\n\n");
    generated.push_str("pub mod limit {\n#[cfg(creusot)]\nuse creusot_std::prelude::*;\n");
    generated.push_str(get("Limit"));
    generated.push('\n');
    generated.push_str(get("Limit::new"));
    generated.push_str("\nimpl<T> Limit<T> {\n");
    generated.push_str("#[cfg(creusot)]\n#[logic(open(self))]\npub(crate) fn inner_logic(self) -> T { self.inner }\n#[cfg(creusot)]\n#[logic(open(self))]\npub(crate) fn limit_logic(self) -> usize { self.limit }\n");
    for name in [
        "Limit::into_inner",
        "Limit::get_ref",
        "Limit::get_mut",
        "Limit::limit",
        "Limit::set_limit",
    ] {
        generated.push_str(get(name));
        generated.push('\n');
    }
    generated.push_str("}\n\n");
    generated.push_str("#[cfg_attr(creusot, ensures(result.0 == inner))]\n#[cfg_attr(creusot, ensures(result.1@ == new_limit@))]\npub fn metadata_round_trip<T>(inner: T, initial_limit: usize, new_limit: usize) -> (T, usize) {\n    let mut value = new(inner, initial_limit);\n    let _ = value.get_ref();\n    value.set_limit(new_limit);\n    let observed = value.limit();\n    let owned = value.into_inner();\n    (owned, observed)\n}\n\npub fn new_for_native<T>(inner: T, limit: usize) -> Limit<T> {\n    new(inner, limit)\n}\n}\n\n");
    generated.push_str("pub mod chain {\n#[cfg(creusot)]\nuse creusot_std::prelude::*;\n");
    generated.push_str(get("Chain"));
    generated.push_str("\nimpl<T, U> Chain<T, U> {\n");
    generated.push_str("#[cfg(creusot)]\n#[logic(open(self))]\npub(crate) fn a_logic(self) -> T { self.a }\n#[cfg(creusot)]\n#[logic(open(self))]\npub(crate) fn b_logic(self) -> U { self.b }\n");
    generated.push_str(get("Chain::new"));
    generated.push('\n');
    generated.push_str(get("Chain::into_inner"));
    generated.push_str("\n}\n}\n");

    for (module, ty, new, get_ref, get_mut, into) in [
        (
            "reader",
            "Reader",
            "Reader::new",
            "Reader::get_ref",
            "Reader::get_mut",
            "Reader::into_inner",
        ),
        (
            "writer",
            "Writer",
            "Writer::new",
            "Writer::get_ref",
            "Writer::get_mut",
            "Writer::into_inner",
        ),
    ] {
        generated.push_str(&format!(
            "\npub mod {module} {{\n#[cfg(creusot)]\nuse creusot_std::prelude::*;\n"
        ));
        generated.push_str(get(ty));
        generated.push('\n');
        generated.push_str(get(new));
        generated.push_str(&format!("\nimpl<B> {ty}<B> {{\n"));
        generated.push_str("#[cfg(creusot)]\n#[logic(open(self))]\npub(crate) fn buf_logic(self) -> B { self.buf }\n");
        for name in [get_ref, get_mut, into] {
            generated.push_str(get(name));
            generated.push('\n');
        }
        generated.push_str("}\n}\n");
    }

    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("actual_adapter_metadata.rs"), generated).unwrap();
    fs::write(out.join("take_source_snapshot.rs"), &take).unwrap();
    fs::write(out.join("limit_source_snapshot.rs"), &limit).unwrap();
    fs::write(out.join("chain_source_snapshot.rs"), &chain).unwrap();
    fs::write(out.join("reader_source_snapshot.rs"), &reader).unwrap();
    fs::write(out.join("writer_source_snapshot.rs"), &writer).unwrap();

    let mut fragments = String::new();
    for (name, source_name, range) in &selected {
        let path = match *source_name {
            "take.rs" => &take_path,
            "limit.rs" => &limit_path,
            "chain.rs" => &chain_path,
            "reader.rs" => &reader_path,
            _ => &writer_path,
        };
        let source = fs::read_to_string(path).unwrap();
        let line = source[..range.1]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1;
        let hash = fnv1a64(range.0.as_bytes());
        fragments.push_str(&format!(
            "{name}: src/buf/{source_name}:{line}, bytes {}..{}, fnv1a64={hash:016x}, {} bytes\n",
            range.1,
            range.2,
            range.0.len()
        ));
    }
    fragments.push_str("Each selected runtime signature/body is copied byte-for-byte before adding cfg(creusot) contracts; exact full-file snapshots are retained. Only adapter field/limit metadata is in scope.\n");
    fs::write(out.join("source_fragments.txt"), fragments).unwrap();
}
