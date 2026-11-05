//! Closed observations of four existing, fully typed ASCII conversion bodies.
use crate::{
    config::{Project, Tool},
    scan,
};
use anyhow::{ensure, Context, Result};
use quote::ToTokens;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path},
    process::Command,
};
const DATA: &str = include_str!("ascii_vec_scope.json");
const DATA46: &str = include_str!("ascii_borrow_scope46.json");
const DATA49: &str = include_str!("ascii_next_scope49.json");
fn scope() -> Result<Value> {
    Ok(serde_json::from_str(DATA)?)
}
fn sha(bytes: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(bytes.as_ref()))
}
fn normal(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path.components().all(|p| matches!(p, Component::Normal(_)))
}
fn inside(root: &Path, relative: &Path) -> Result<std::path::PathBuf> {
    ensure!(
        normal(relative),
        "Typed ASCII path must stay inside package"
    );
    let root = root.canonicalize()?;
    let mut cursor = root.clone();
    for part in relative.components() {
        cursor.push(part.as_os_str());
        ensure!(
            !fs::symlink_metadata(&cursor)?.file_type().is_symlink(),
            "Typed ASCII source/ancestor symlink rejected"
        );
    }
    let out = cursor.canonicalize()?;
    ensure!(out.starts_with(&root), "Typed ASCII path escapes package");
    Ok(out)
}
fn map_files(root: &Path, files: &Value) -> Result<()> {
    for (name, digest) in files.as_object().context("Missing fixed file closure")? {
        let path = inside(root, Path::new(name))?;
        ensure!(
            Some(sha(fs::read(path)?).as_str()) == digest.as_str(),
            "Typed ASCII fixed source/proof/evidence mismatch: {name}"
        );
    }
    Ok(())
}
fn closed_inputs(root: &Path, fixed: &Value) -> Result<()> {
    ensure!(
        !root.join("build.rs").exists(),
        "Typed ASCII unlisted automatic build.rs rejected"
    );
    // Cargo's source/control discovery is part of the immutable input boundary.
    // Artifact proof/evidence/tools directories are not Cargo source targets.
    fn rust_files(base: &Path, dir: &Path, out: &mut BTreeSet<String>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let rel = path.strip_prefix(base)?;
            if rel.components().next().is_some_and(|p| {
                matches!(
                    p.as_os_str().to_str(),
                    Some("proofs" | "evidence" | "tools" | "target" | ".git" | ".lake")
                )
            }) {
                continue;
            }
            ensure!(
                !entry.file_type()?.is_symlink(),
                "Typed ASCII unlisted source symlink rejected"
            );
            if entry.file_type()?.is_dir() {
                rust_files(base, &path, out)?;
            } else if path.extension().is_some_and(|x| x == "rs") {
                out.insert(rel.to_string_lossy().into_owned());
            }
        }
        Ok(())
    }
    let mut actual = BTreeSet::new();
    rust_files(root, root, &mut actual)?;
    let expected: BTreeSet<String> = fixed["archive_files"]
        .as_object()
        .context("Missing original archive fileset")?
        .keys()
        .filter(|p| p.ends_with(".rs"))
        .cloned()
        .collect();
    ensure!(
        actual == expected,
        "Typed ASCII Rust source fileset differs from original archive"
    );
    for ancestor in root.ancestors() {
        for name in [".cargo/config", ".cargo/config.toml"] {
            ensure!(
                !ancestor.join(name).exists(),
                "Typed ASCII Cargo ancestor configuration rejected"
            );
        }
    }
    Ok(())
}
pub(crate) fn keys() -> Result<BTreeSet<String>> {
    Ok(scope()?["bindings"]
        .as_array()
        .context("Missing fixed bindings")?
        .iter()
        .map(|b| b["proof"]["api"].as_str().unwrap().to_owned())
        .collect())
}
/// Checks structural source facts independently of the outer whole-file digests.
/// No missing callable is invented: all four come from the dedicated AST scanner.
pub(crate) fn semantic_contracts(project: &Project) -> Result<Vec<scan::Contract>> {
    let fixed = scope()?;
    let package = project.manifest.parent().context("Missing package")?;
    let path = inside(package, Path::new("src/ascii_string.rs"))?;
    let text = fs::read_to_string(&path)?;
    let parsed = syn::parse_file(&text)?;
    let expected = syn::parse_file(fixed["ascii_string_imports"].as_str().unwrap())?;
    let uses = |file: &syn::File| {
        file.items
            .iter()
            .filter_map(|i| {
                if let syn::Item::Use(u) = i {
                    Some(u.to_token_stream().to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };
    ensure!(
        uses(&parsed) == uses(&expected),
        "Typed ASCII imports differ from the original resolved source chain"
    );
    let found = scan::ascii_vec_public_contracts(project, &keys()?)?;
    ensure!(found.len() == 4, "Expected four typed ASCII source bodies");
    for binding in fixed["bindings"].as_array().unwrap() {
        let key = binding["proof"]["api"].as_str().unwrap();
        let matches: Vec<_> = found
            .iter()
            .filter(|c| c.api_paths.contains(&key.to_owned()))
            .collect();
        ensure!(
            matches.len() == 1,
            "Typed ASCII callable must resolve exactly once: {key}"
        );
        let c = matches[0];
        ensure!(
            c.file.canonicalize()? == path
                && Some(c.first_line as u64) == binding["first_line"].as_u64()
                && Some(c.last_line as u64) == binding["last_line"].as_u64(),
            "Typed ASCII original body source span mismatch"
        );
        let expected: syn::ImplItemFn = syn::parse_str(binding["method_source"].as_str().unwrap())?;
        let actual: Vec<_> = parsed
            .items
            .iter()
            .filter_map(|i| {
                if let syn::Item::Impl(i) = i {
                    Some(i)
                } else {
                    None
                }
            })
            .flat_map(|i| {
                i.items.iter().filter_map(move |m| {
                    if let syn::ImplItem::Fn(f) = m {
                        Some((i, f))
                    } else {
                        None
                    }
                })
            })
            .filter(|(_, f)| {
                f.sig.ident == expected.sig.ident && f.span().start().line == c.first_line
            })
            .collect();
        ensure!(
            actual.len() == 1,
            "Typed ASCII original method/header ambiguous"
        );
        let (implementation, actual) = actual[0];
        ensure!(
            implementation.generics.params.is_empty()
                && implementation.generics.where_clause.is_none()
                && implementation.unsafety.is_none()
                && implementation.defaultness.is_none(),
            "Typed ASCII impl generic/predicate/safety header changed"
        );
        ensure!(
            actual.sig.unsafety.is_some() == binding["unsafe"].as_bool().unwrap(),
            "Typed ASCII unsafe header mismatch"
        );
        ensure!(
            actual.sig.to_token_stream().to_string() == expected.sig.to_token_stream().to_string(),
            "Typed ASCII receiver/return/generic signature mismatch"
        );
        ensure!(
            actual.block.to_token_stream().to_string()
                == expected.block.to_token_stream().to_string(),
            "Typed ASCII original body closure changed"
        );
    }
    Ok(found)
}
fn borrow_contracts(project: &Project, fixed: &Value) -> Result<Vec<scan::Contract>> {
    let package = project.manifest.parent().context("Missing package")?;
    let bindings = fixed["borrow_bindings"].as_array().context("Missing fixed borrow bindings")?;
    let requested = bindings.iter().map(|b| b["proof"]["api"].as_str().unwrap().to_owned()).collect();
    let found = match bindings.len() {
        5 => scan::ascii_borrow_public_contracts(project, &requested)?,
        6 => scan::ascii_borrow_six_public_contracts(project, &requested)?,
        _ => anyhow::bail!("Unknown finite ASCII borrow group"),
    };
    ensure!(found.len() == bindings.len(), "Wrong finite ASCII borrow body count");
    for binding in bindings {
        let file = binding["file"].as_str().unwrap();
        let path = inside(package, Path::new(file))?;
        let parsed = syn::parse_file(&fs::read_to_string(&path)?)?;
        let expected_imports = syn::parse_file(fixed["borrow_imports"][file].as_str().context("Missing fixed borrow imports")?)?;
        let uses = |f: &syn::File| f.items.iter().filter_map(|i| if let syn::Item::Use(u) = i { Some(u.to_token_stream().to_string()) } else { None }).collect::<Vec<_>>();
        ensure!(uses(&parsed) == uses(&expected_imports), "ASCII borrow original import chain changed");
        let key = binding["proof"]["api"].as_str().unwrap();
        let matches: Vec<_> = found
            .iter()
            .filter(|c| c.api_paths.contains(&key.to_owned()))
            .collect();
        ensure!(
            matches.len() == 1,
            "Typed ASCII callable must resolve exactly once: {key}"
        );
        let c = matches[0];
        ensure!(
            c.file.canonicalize()? == path
                && Some(c.first_line as u64) == binding["first_line"].as_u64()
                && Some(c.last_line as u64) == binding["last_line"].as_u64(),
            "Typed ASCII original body source span mismatch"
        );
        let expected: syn::ImplItemFn = syn::parse_str(binding["method_source"].as_str().unwrap())?;
        let actual: Vec<_> = parsed
            .items
            .iter()
            .filter_map(|i| {
                if let syn::Item::Impl(i) = i {
                    Some(i)
                } else {
                    None
                }
            })
            .flat_map(|i| {
                i.items.iter().filter_map(move |m| {
                    if let syn::ImplItem::Fn(f) = m {
                        Some((i, f))
                    } else {
                        None
                    }
                })
            })
            .filter(|(_, f)| {
                f.sig.ident == expected.sig.ident && f.span().start().line == c.first_line
            })
            .collect();
        ensure!(
            actual.len() == 1,
            "Typed ASCII original method/header ambiguous"
        );
        let (implementation, actual) = actual[0];
        ensure!(
            implementation.generics.params.is_empty()
                && implementation.generics.where_clause.is_none()
                && implementation.unsafety.is_none()
                && implementation.defaultness.is_none(),
            "Typed ASCII impl generic/predicate/safety header changed"
        );
        ensure!(
            actual.sig.unsafety.is_some() == binding["unsafe"].as_bool().unwrap(),
            "Typed ASCII unsafe header mismatch"
        );
        ensure!(
            actual.sig.to_token_stream().to_string() == expected.sig.to_token_stream().to_string(),
            "Typed ASCII receiver/return/generic signature mismatch"
        );
        ensure!(
            actual.block.to_token_stream().to_string()
                == expected.block.to_token_stream().to_string(),
            "Typed ASCII original body closure changed"
        );
    }
    Ok(found)
}
pub(crate) fn discover(project: &Project, tool: &Tool) -> Result<Vec<scan::Contract>> {
    ensure!(
        project.name == "ascii" && project.version == "1.1.0",
        "Typed ASCII bindings support only ascii1.1.0"
    );
    ensure!(
        tool.exact_source_bindings.is_none() && tool.external_blanket_observations.is_none(),
        "Typed ASCII bindings cannot combine other source opt-ins"
    );
    let package = project
        .manifest
        .parent()
        .context("Missing package")?
        .canonicalize()?;
    ensure!(
        project.root_source.canonicalize()? == inside(&package, Path::new("src/lib.rs"))?,
        "Typed ASCII library root mismatch"
    );
    // Only three compiled audited snapshots; manifest data cannot define a new scope.
    let manifest_path = inside(&package, tool.ascii_vec_source_bindings.as_deref().context("Missing typed ASCII opt-in")?)?;
    let manifest: Value = serde_json::from_slice(&fs::read(manifest_path)?)?;
    let digest = manifest["fixed_scope_sha256"].as_str().context("Missing fixed ASCII scope digest")?;
    let (fixed, data, snapshot) = if digest == sha(DATA) { (scope()?, DATA, 41) }
        else if digest == sha(DATA46) { (serde_json::from_str::<Value>(DATA46)?, DATA46, 46) }
        else if digest == sha(DATA49) { (serde_json::from_str::<Value>(DATA49)?, DATA49, 49) }
        else { anyhow::bail!("Unknown compiled ASCII source snapshot") };
    closed_inputs(&package, &fixed)?;
    ensure!(
        sha(fs::read(inside(
            &package,
            Path::new("evidence/upstream.crate")
        )?)?)
            == fixed["archive_sha256"].as_str().unwrap(),
        "Typed ASCII original archive digest mismatch"
    );
    ensure!(
        tool.lean_dir.as_deref() == Some(Path::new("proofs")),
        "Typed ASCII proof directory must retain the audited closure"
    );
    ensure!(
        project.features == BTreeSet::from(["alloc".into(), "default".into(), "std".into()]),
        "Typed ASCII requires original default std/alloc configuration"
    );
    let path = inside(
        &package,
        tool.ascii_vec_source_bindings
            .as_deref()
            .context("Missing typed ASCII opt-in")?,
    )?;
    let manifest: Value = serde_json::from_slice(&fs::read(path)?)?;
    let expected = json!({"schema":1,"crate":"ascii","version":"1.1.0","kind":"local_typed_impl","local_declaration":true,"fixed_scope_sha256":sha(data),"keys":fixed["bindings"].as_array().unwrap().iter().chain(fixed["borrow_bindings"].as_array().into_iter().flatten()).map(|b|b["proof"]["api"].clone()).collect::<Vec<_>>()});
    ensure!(
        manifest == expected,
        "Typed ASCII finite manifest/scope/keys mismatch"
    );
    ensure!(
        tool.version == fixed["aeneas_version"]
            && serde_json::to_value(&tool.proofs)? == fixed[format!("proofs{snapshot}")],
        "Typed ASCII requires exact audited snapshot mappings/theorems/preconditions"
    );
    // First establish typed body identity; outer source hashes cannot mask semantic
    // owner/trait/safety/body failures in focused mutation tests.
    let mut found = semantic_contracts(project)?;
    if snapshot >= 46 { found.extend(borrow_contracts(project, &fixed)?); }
    let selected: BTreeSet<String> = found.iter().flat_map(|c|c.api_paths.iter().filter(|k| fixed["bindings"].as_array().unwrap().iter().chain(fixed["borrow_bindings"].as_array().into_iter().flatten()).any(|b| b["proof"]["api"].as_str() == Some(k.as_str()))).cloned()).collect();
    map_files(&package, &fixed["source_files"])?;
    map_files(&package, &fixed["proof_files"])?;
    map_files(&package, &fixed["evidence_files"])?;
    let rustc = Command::new("rustc").arg("-vV").output()?;
    ensure!(
        rustc.status.success(),
        "Cannot inspect typed ASCII compiler"
    );
    let rustc = String::from_utf8(rustc.stdout)?;
    ensure!(
        rustc
            .lines()
            .any(|s| s == "commit-hash: 923c95cdf5ba65cea505aa2ea829f578e1506ed8")
            && rustc.lines().any(|s| s == "host: x86_64-unknown-linux-gnu"),
        "Typed ASCII compiler/host mismatch"
    );
    let cfg = Command::new("rustc").args(["--print", "cfg"]).output()?;
    ensure!(cfg.status.success(), "Cannot inspect typed ASCII target");
    let cfg: BTreeSet<String> = String::from_utf8(cfg.stdout)?
        .lines()
        .map(String::from)
        .collect();
    ensure!(cfg == project.cfg, "Typed ASCII target cfg mismatch");
    eprintln!("Typed ASCII source gate: {} actual original typed bodies; compiled fixed{} source/proof closure; old37 scanner mode retained", found.len(), snapshot);
    Ok(found
        .into_iter()
        .map(|mut c| {
            let k = c
                .api_paths
                .iter()
                .find(|k| selected.contains(*k))
                .unwrap()
                .clone();
            c.api_paths = vec![k];
            c
        })
        .collect())
}
use syn::spanned::Spanned;

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (tempfile::TempDir, Project) {
        let base = std::path::PathBuf::from(std::env::var("PROOFS_ASCII41_ROOT").unwrap());
        let dir = tempfile::tempdir().unwrap();
        let fixed = scope().unwrap();
        for name in fixed["source_files"].as_object().unwrap().keys() {
            let out = dir.path().join(name);
            fs::create_dir_all(out.parent().unwrap()).unwrap();
            fs::copy(base.join(name), out).unwrap();
        }
        let p = Project {
            name: "ascii".into(),
            version: "1.1.0".into(),
            lib_name: "ascii".into(),
            manifest: dir.path().join("Cargo.toml"),
            root_source: dir.path().join("src/lib.rs"),
            features: BTreeSet::from(["alloc".into(), "default".into(), "std".into()]),
            cfg: BTreeSet::new(),
        };
        (dir, p)
    }
    #[test]
    #[ignore = "Original-source finite borrow-six delta semantic checks; no compiler/server"]
    fn borrow_six_delta_source_mutations() {
        let (_dir, p) = fixture();
        let fixed: Value = serde_json::from_str(DATA49).unwrap();
        assert_eq!(borrow_contracts(&p, &fixed).unwrap().len(), 6);
        let path = p.manifest.parent().unwrap().join("src/ascii_str.rs");
        let original = fs::read_to_string(&path).unwrap();
        for (name, before, after) in [
            ("self-owner", "impl AsMut<AsciiStr> for AsciiStr {", "impl AsMut<AsciiStr> for Vec<AsciiChar> {"),
            ("self-target-u8", "impl AsMut<AsciiStr> for AsciiStr {", "impl AsMut<[u8]> for AsciiStr {"),
            ("self-target-slice", "impl AsMut<AsciiStr> for AsciiStr {", "impl AsMut<[AsciiChar]> for AsciiStr {"),
            ("self-mutable-receiver", "fn as_mut(&mut self) -> &mut AsciiStr {\n        self", "fn as_mut(&self) -> &mut AsciiStr {\n        self"),
            ("self-writeback-body", "fn as_mut(&mut self) -> &mut AsciiStr {\n        self", "fn as_mut(&mut self) -> &mut AsciiStr {\n        self.other"),
            ("self-bound", "impl AsMut<AsciiStr> for AsciiStr {", "impl AsMut<AsciiStr> for AsciiStr where AsciiStr: Clone {"),
        ] {
            assert!(original.contains(before));
            fs::write(&path, original.replacen(before, after, 1)).unwrap();
            assert!(borrow_contracts(&p, &fixed).is_err(), "Accepted self AsMut mutation {name}");
            fs::write(&path, &original).unwrap();
            eprintln!("borrow-six-semantic-rejected:{name}");
        }
        assert_eq!(borrow_contracts(&p, &fixed).unwrap().len(), 6);
    }
    #[test]
    #[ignore = "Original-source finite borrow-five semantic matrix; no compiler/server"]
    fn borrow_five_inner_source_mutations() {
        let (_dir, p) = fixture();
        let fixed: Value = serde_json::from_str(DATA46).unwrap();
        assert_eq!(borrow_contracts(&p, &fixed).unwrap().len(), 5);
        let cases = [
            ("src/ascii_str.rs", "slice-u8", "impl AsRef<[AsciiChar]> for AsciiStr", "impl AsRef<[u8]> for AsciiStr"),
            ("src/ascii_str.rs", "owner-string", "impl AsRef<[AsciiChar]> for AsciiStr", "impl AsRef<[AsciiChar]> for AsciiString"),
            ("src/ascii_str.rs", "target-vec", "impl AsRef<[AsciiChar]> for AsciiStr", "impl AsRef<Vec<AsciiChar>> for AsciiStr"),
            ("src/ascii_str.rs", "self-target", "impl AsRef<AsciiStr> for AsciiStr", "impl AsRef<AsciiString> for AsciiStr"),
            ("src/ascii_str.rs", "mutable-receiver", "fn as_mut(&mut self) -> &mut [AsciiChar]", "fn as_mut(&self) -> &mut [AsciiChar]"),
            ("src/ascii_str.rs", "borrow-body", "fn as_mut(&mut self) -> &mut [AsciiChar] {\n        &mut self.slice", "fn as_mut(&mut self) -> &mut [AsciiChar] {\n        &mut self.other"),
            ("src/ascii_str.rs", "extra-bound", "impl AsMut<[AsciiChar]> for AsciiStr {", "impl AsMut<[AsciiChar]> for AsciiStr where AsciiStr: Clone {"),
            ("src/ascii_string.rs", "string-slice-u8", "impl AsMut<[AsciiChar]> for AsciiString", "impl AsMut<[u8]> for AsciiString"),
            ("src/ascii_string.rs", "string-owner-vec", "impl AsRef<[AsciiChar]> for AsciiString", "impl AsRef<[AsciiChar]> for Vec<AsciiChar>"),
            ("src/ascii_string.rs", "writeback-body", "fn as_mut(&mut self) -> &mut [AsciiChar] {\n        &mut self.vec", "fn as_mut(&mut self) -> &mut [AsciiChar] {\n        &mut self.other"),
            ("src/ascii_string.rs", "unsafe-header", "fn as_ref(&self) -> &[AsciiChar]", "unsafe fn as_ref(&self) -> &[AsciiChar]"),
        ];
        for (file, name, before, after) in cases {
            let path = p.manifest.parent().unwrap().join(file);
            let original = fs::read_to_string(&path).unwrap();
            assert!(original.contains(before), "Missing fixture {name}");
            fs::write(&path, original.replacen(before, after, 1)).unwrap();
            assert!(borrow_contracts(&p, &fixed).is_err(), "Accepted borrow mutation {name}");
            fs::write(&path, original).unwrap();
            eprintln!("borrow-semantic-rejected:{name}");
        }
        assert_eq!(borrow_contracts(&p, &fixed).unwrap().len(), 5);
    }
    #[test]
    #[ignore = "Original-source semantic matrix; no compiler/server"]
    fn thirty_two_inner_source_mutations() {
        let (_dir, p) = fixture();
        let path = p.manifest.parent().unwrap().join("src/ascii_string.rs");
        let original = fs::read_to_string(&path).unwrap();
        assert_eq!(semantic_contracts(&p).unwrap().len(), 4);
        let cases=[
   ("owner-u8","impl From<AsciiString> for Vec<AsciiChar>","impl From<AsciiString> for Vec<u8>"),
   ("trait-arg-u8","impl From<Vec<AsciiChar>> for AsciiString","impl From<Vec<u8>> for AsciiString"),
   ("checked-owner-u8","impl IntoAsciiString for Vec<AsciiChar>","impl IntoAsciiString for Vec<u8>"),
   ("unknown-char","impl IntoAsciiString for Vec<AsciiChar>","impl IntoAsciiString for Vec<OtherChar>"),
   ("allocator","impl IntoAsciiString for Vec<AsciiChar>","impl IntoAsciiString for Vec<AsciiChar, OtherAllocator>"),
   ("array-owner","impl From<Vec<AsciiChar>> for AsciiString","impl From<[AsciiChar; 1]> for AsciiString"),
   ("reference-owner","impl IntoAsciiString for Vec<AsciiChar>","impl IntoAsciiString for &Vec<AsciiChar>"),
   ("wrong-trait","impl From<Vec<AsciiChar>> for AsciiString","impl Into<Vec<AsciiChar>> for AsciiString"),
   ("checked-name","fn into_ascii_string(self) -> Result<AsciiString, FromAsciiError<Self>> {\n        Ok(AsciiString::from(self))","fn not_checked(self) -> Result<AsciiString, FromAsciiError<Self>> {\n        Ok(AsciiString::from(self))"),
   ("unsafe-removed","unsafe fn into_ascii_string_unchecked(self) -> AsciiString {\n        AsciiString::from(self)","fn into_ascii_string_unchecked(self) -> AsciiString {\n        AsciiString::from(self)"),
   ("unsafe-added","fn into_ascii_string(self) -> Result<AsciiString, FromAsciiError<Self>> {\n        Ok(AsciiString::from(self))","unsafe fn into_ascii_string(self) -> Result<AsciiString, FromAsciiError<Self>> {\n        Ok(AsciiString::from(self))"),
   ("unwrap-body","fn from(s: AsciiString) -> Vec<AsciiChar> {\n        s.vec","fn from(s: AsciiString) -> Vec<AsciiChar> {\n        Vec::new()"),
   ("checked-body","Ok(AsciiString::from(self))","Err(AsciiString::from(self))"),
   ("unchecked-body","unsafe fn into_ascii_string_unchecked(self) -> AsciiString {\n        AsciiString::from(self)","unsafe fn into_ascii_string_unchecked(self) -> AsciiString {\n        AsciiString::new()"),
   ("argument-name","fn from(vec: Vec<AsciiChar>) -> Self","fn from(other: Vec<AsciiChar>) -> Self"),
   ("return-type","fn from(vec: Vec<AsciiChar>) -> Self","fn from(vec: Vec<AsciiChar>) -> AsciiStr"),
   ("receiver-reference","fn into_ascii_string(self) -> Result<AsciiString, FromAsciiError<Self>> {\n        Ok(AsciiString::from(self))","fn into_ascii_string(&self) -> Result<AsciiString, FromAsciiError<Self>> {\n        Ok(AsciiString::from(self))"),
   ("extra-where","impl From<Vec<AsciiChar>> for AsciiString {","impl From<Vec<AsciiChar>> for AsciiString where AsciiString: Clone {"),
   ("extra-generic","impl From<Vec<AsciiChar>> for AsciiString {","impl<T> From<Vec<AsciiChar>> for AsciiString {"),
   ("unsafe-impl","impl IntoAsciiString for Vec<AsciiChar> {","unsafe impl IntoAsciiString for Vec<AsciiChar> {"),
   ("negative-impl","impl IntoAsciiString for Vec<AsciiChar> {","impl !IntoAsciiString for Vec<AsciiChar> {"),
   ("unknown-Vec-import","use alloc::vec::Vec;","use fake::Vec;"),
   ("unknown-char-import","use ascii_char::AsciiChar;","use fake::AsciiChar;"),
   ("aliased-char-import","use ascii_char::AsciiChar;","use ascii_char::AsciiChar as OtherChar;"),
   ("direction-swap","impl From<AsciiString> for Vec<AsciiChar>","impl From<Vec<AsciiChar>> for AsciiString"),
   ("borrowed-from-argument","fn from(s: AsciiString) -> Vec<AsciiChar>","fn from(s: &AsciiString) -> Vec<AsciiChar>"),
   ("source-span","impl From<Vec<AsciiChar>> for AsciiString {","\nimpl From<Vec<AsciiChar>> for AsciiString {"),
   ("wrong-error-type","fn into_ascii_string(self) -> Result<AsciiString, FromAsciiError<Self>> {\n        Ok(AsciiString::from(self))","fn into_ascii_string(self) -> Result<AsciiString, ()> {\n        Ok(AsciiString::from(self))"),
   ("method-generic","fn from(vec: Vec<AsciiChar>) -> Self","fn from<T>(vec: Vec<AsciiChar>) -> Self")
  ];
        assert_eq!(cases.len(), 29);
        for (name, before, after) in cases {
            assert!(original.contains(before), "Fixture lacks {name}");
            fs::write(&path, original.replacen(before, after, 1)).unwrap();
            assert!(
                semantic_contracts(&p).is_err(),
                "Semantic validator accepted {name}"
            );
            fs::write(&path, &original).unwrap();
            eprintln!("semantic-rejected:{name}");
        }
        let manifest = fs::read_to_string(&p.manifest).unwrap();
        for edition in ["2018", "2021", "2024"] {
            let altered = manifest.replacen(
                "[package]",
                &format!("[package]\nedition = \"{edition}\""),
                1,
            );
            fs::write(&p.manifest, altered).unwrap();
            assert!(semantic_contracts(&p).is_err());
            fs::write(&p.manifest, &manifest).unwrap();
            eprintln!("semantic-rejected:edition-{edition}");
        }
        assert_eq!(semantic_contracts(&p).unwrap().len(), 4);
        eprintln!("32 inner semantic mutations rejected before whole-source digest validation");
    }
    #[test]
    #[ignore = "Immutable public old37 config comparison; no server"]
    fn none_configuration_matches_immutable_old37() {
        let mut args = crate::ProjectArgs::default();
        args.manifest_path = Some(std::env::var("PROOFS_ASCII37_MANIFEST").unwrap().into());
        let p = Project::load(&args).unwrap();
        let config = p.config().unwrap();
        assert_eq!(config.tool.proofs.len(), 37);
        assert!(config.tool.ascii_vec_source_bindings.is_none());
        let v = crate::aeneas::configuration(&config.tool).unwrap();
        assert!(v.get("ascii_vec_source_bindings").is_none());
        assert!(serde_json::to_value(&config.tool)
            .unwrap()
            .get("ascii_vec_source_bindings")
            .is_none());
        let expected: Value = serde_json::from_slice(
            &fs::read(std::env::var("PROOFS_ASCII37_CONFIG").unwrap()).unwrap(),
        )
        .unwrap();
        let actual = serde_json::to_vec(&v).unwrap();
        assert_eq!(actual, serde_json::to_vec(&expected).unwrap());
        fs::write(std::env::var("PROOFS_ASCII37_CONFIG_OUT").unwrap(), &actual).unwrap();
        eprintln!(
            "None configuration bytes exactly equal old37 immutable SARIF serde bytes: {}",
            sha(actual)
        );
    }
}

#[cfg(test)]
mod captured_tests {
    use super::*;
    fn git(root: &Path, args: &[&str]) -> String {
        crate::git::run(root, args).unwrap()
    }
    #[test]
    #[ignore = "Private local Git recorded-source/cfg/symlink tests; no server or Lean build"]
    fn recorded_source_and_closure_controls() {
        captured_case(std::path::PathBuf::from(std::env::var("PROOFS_ASCII41_ROOT").unwrap()), scope().unwrap(), 41);
        if let Ok(base) = std::env::var("PROOFS_ASCII46_ROOT") {
            captured_case(base.into(), serde_json::from_str(DATA46).unwrap(), 46);
        }
        if let Ok(base) = std::env::var("PROOFS_ASCII49_ROOT") {
            captured_case(base.into(), serde_json::from_str(DATA49).unwrap(), 49);
        }
    }
    fn captured_case(base: std::path::PathBuf, fixed: Value, expected_count: usize) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let mut names: BTreeSet<String> = fixed["source_files"]
            .as_object()
            .unwrap()
            .keys()
            .chain(fixed["proof_files"].as_object().unwrap().keys())
            .chain(fixed["evidence_files"].as_object().unwrap().keys())
            .cloned()
            .collect();
        names.extend([
            "proofs.toml".into(),
            "evidence/upstream.crate".into(),
            "evidence/ascii-vec-source-bindings.json".into(),
        ]);
        for name in &names {
            let dest = root.join(name);
            fs::create_dir_all(dest.parent().unwrap()).unwrap();
            fs::copy(base.join(name), dest).unwrap();
        }
        let mut args = crate::ProjectArgs::default();
        args.manifest_path = Some(root.join("Cargo.toml"));
        let project = Project::load(&args).unwrap();
        let mut config = project.config().unwrap();
        let original = crate::aeneas::discover(&project, &config.tool).unwrap();
        assert_eq!(original.len(), expected_count);
        // Real unlisted control/source inputs must be rejected even when inert.
        for (name, body) in [
            ("build.rs", "fn main() {}"),
            ("src/unused_extra.rs", "// inert"),
            (".cargo/config.toml", "[build]\njobs=1\n"),
        ] {
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, body).unwrap();
            assert!(discover(&project, &config.tool).is_err(), "Accepted {name}");
            fs::remove_file(path).unwrap();
            if name.starts_with(".cargo/") {
                fs::remove_dir(root.join(".cargo")).unwrap();
            }
        }
        let proof = root.join("proofs/VectorIdentityProofs.lean");
        let proof_bytes = fs::read(&proof).unwrap();
        fs::write(&proof, b"-- wrong theorem closure").unwrap();
        assert!(discover(&project, &config.tool).is_err());
        fs::write(&proof, proof_bytes).unwrap();
        let data = root.join("evidence/ascii-vec-source-bindings.json");
        let data_bytes = fs::read(&data).unwrap();
        fs::write(&data, b"{}").unwrap();
        assert!(discover(&project, &config.tool).is_err());
        fs::write(&data, &data_bytes).unwrap();
        git(root, &["init", "-q"]);
        git(root, &["config", "user.name", "CLI test"]);
        git(root, &["config", "user.email", "test@example.invalid"]);
        git(root, &["add", "-f", "."]);
        git(
            root,
            &[
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-qm",
                "Immutable ASCII source",
            ],
        );
        let commit = git(root, &["rev-parse", "HEAD"]);
        let mut record = crate::record::Record {
            sarif: json!({"runs":[{"versionControlProvenance":[{"revisionId":commit}]}]}),
            contracts: vec![],
        };
        fs::write(root.join("metadata-only.txt"), b"After verification").unwrap();
        git(root, &["add", "metadata-only.txt"]);
        git(
            root,
            &[
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-qm",
                "Metadata only",
            ],
        );
        assert_ne!(git(root, &["rev-parse", "HEAD"]), commit);
        let source = root.join("src/ascii_string.rs");
        let source_bytes = fs::read(&source).unwrap();
        fs::write(&source, b"// mutable current HEAD ignored").unwrap();
        let captured =
            crate::external_blanket::discover_recorded(&project, &args, &config.tool, &record)
                .unwrap();
        assert_eq!(captured.len(), expected_count);
        assert_eq!(
            original.iter().map(|c| &c.api_paths).collect::<Vec<_>>(),
            captured.iter().map(|c| &c.api_paths).collect::<Vec<_>>()
        );
        fs::write(&source, &source_bytes).unwrap();
        let saved = config.tool.proofs[40].precondition.clone();
        config.tool.proofs[40].precondition.push_str(" changed");
        assert!(
            crate::external_blanket::discover_recorded(&project, &args, &config.tool, &record)
                .is_err()
        );
        config.tool.proofs[40].precondition = saved;
        // A different captured source must fail instead of silently using current data.
        fs::write(&source, b"// wrong immutable source").unwrap();
        git(root, &["add", "src/ascii_string.rs"]);
        git(
            root,
            &[
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-qm",
                "Wrong captured source",
            ],
        );
        record.sarif["runs"][0]["versionControlProvenance"][0]["revisionId"] =
            json!(git(root, &["rev-parse", "HEAD"]));
        fs::write(&source, &source_bytes).unwrap();
        assert!(
            crate::external_blanket::discover_recorded(&project, &args, &config.tool, &record)
                .is_err()
        );
        let manifest = root.join("Cargo.toml");
        let manifest_bytes = fs::read(&manifest).unwrap();
        let external = tempfile::NamedTempFile::new().unwrap();
        fs::write(external.path(), &manifest_bytes).unwrap();
        fs::remove_file(&manifest).unwrap();
        std::os::unix::fs::symlink(external.path(), &manifest).unwrap();
        git(root, &["add", "Cargo.toml"]);
        git(
            root,
            &[
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-qm",
                "Escaping captured manifest",
            ],
        );
        record.sarif["runs"][0]["versionControlProvenance"][0]["revisionId"] =
            json!(git(root, &["rev-parse", "HEAD"]));
        fs::remove_file(&manifest).unwrap();
        fs::write(&manifest, &manifest_bytes).unwrap();
        assert!(
            crate::external_blanket::discover_recorded(&project, &args, &config.tool, &record)
                .unwrap_err()
                .to_string()
                .contains("symlink")
        );
        assert_eq!(
            git(root, &["worktree", "list", "--porcelain"])
                .lines()
                .filter(|s| s.starts_with("worktree "))
                .count(),
            1
        );
        record.sarif["runs"][0]["versionControlProvenance"][0]["revisionId"] = json!("badcommit");
        assert!(
            crate::external_blanket::discover_recorded(&project, &args, &config.tool, &record)
                .is_err()
        );
        eprintln!("Captured ASCII{expected_count} immutable commit succeeds after metadata HEAD advance/live source edit; wrong config/source/symlink/commit rejected; worktree cleanup and extra source/control/theorem/data closure checked");
    }
}
