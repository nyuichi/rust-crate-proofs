//! Narrow, opt-in compiler observations. Author metadata never creates callables.
use crate::config::{Project, Tool};
use anyhow::{ensure, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::process::Command;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path},
};

const CORE_CHECKSUM: &str = "63b8176103e19a2643978565ca18b50549f6101881c443590420e4dc998a3c69";
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Manifest {
    schema: u32,
    kind: String,
    local_declaration: bool,
    #[serde(rename = "crate")]
    crate_name: String,
    version: String,
    target: String,
    compiler: String,
    dependency: Dependency,
    configurations: BTreeMap<String, Vec<String>>,
    keys: Vec<String>,
    source_files: BTreeMap<String, String>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Dependency {
    name: String,
    version: String,
    checksum: String,
}
fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path.components().all(|p| matches!(p, Component::Normal(_)))
}
fn expected_keys() -> BTreeSet<String> {
    let names = [
        "SplitMix64",
        "Xoroshiro64Star",
        "Xoroshiro64StarStar",
        "Xoroshiro128Plus",
        "Xoroshiro128PlusPlus",
        "Xoroshiro128StarStar",
        "Xoshiro128Plus",
        "Xoshiro128PlusPlus",
        "Xoshiro128StarStar",
        "Xoshiro256Plus",
        "Xoshiro256PlusPlus",
        "Xoshiro256StarStar",
        "Xoshiro512Plus",
        "Xoshiro512PlusPlus",
        "Xoshiro512StarStar",
    ];
    names
        .into_iter()
        .flat_map(|name| {
            let widths: &[u8] = if name.starts_with("Xoroshiro64") || name.starts_with("Xoshiro128")
            {
                &[32]
            } else {
                &[32, 64]
            };
            widths.iter().map(move |width| {
                format!("<rand_xoshiro::{name} as rand_core::Rng>::next_u{width}")
            })
        })
        .collect()
}
fn validate_manifest(m: &Manifest) -> Result<()> {
    ensure!(
        m.kind == "external_blanket_specialization" && !m.local_declaration,
        "External specialization must not claim local declaration"
    );
    ensure!(
        m.schema == 1 && m.crate_name == "rand_xoshiro" && m.version == "0.8.1",
        "Unsupported external blanket scope"
    );
    ensure!(
        m.target == "x86_64-unknown-linux-gnu" && m.compiler == "nightly-2026-09-17",
        "Unsupported external blanket compiler/target"
    );
    ensure!(
        m.dependency.name == "rand_core"
            && m.dependency.version == "0.10.1"
            && m.dependency.checksum == CORE_CHECKSUM,
        "External blanket dependency mismatch"
    );
    ensure!(
        m.configurations
            == BTreeMap::from([
                ("default".into(), vec![]),
                ("serde".into(), vec!["serde".into()])
            ]),
        "External blanket requires exactly default and serde configurations"
    );
    ensure!(
        m.keys.len() == 25 && m.keys.iter().cloned().collect::<BTreeSet<_>>() == expected_keys(),
        "External blanket requires exactly the audited 25 fully qualified keys"
    );
    ensure!(
        !m.source_files.is_empty(),
        "Missing original source closure"
    );
    for (path, hash) in &m.source_files {
        ensure!(
            relative(Path::new(path))
                && hash.len() == 64
                && hash.bytes().all(|b| b.is_ascii_hexdigit()),
            "Invalid external blanket source path/digest"
        );
    }
    Ok(())
}
pub(crate) fn validate_scope(project: &Project, tool: &Tool) -> Result<Manifest> {
    ensure!(
        project.name == "rand_xoshiro" && project.version == "0.8.1",
        "External blanket observations support only rand_xoshiro 0.8.1"
    );
    ensure!(
        tool.exact_source_bindings.is_none(),
        "External blanket observations cannot reuse exact-source selection"
    );
    let relative_path = tool
        .external_blanket_observations
        .as_deref()
        .context("Missing external blanket opt-in")?;
    ensure!(
        relative(relative_path),
        "External blanket manifest must remain inside package"
    );
    let package = project
        .manifest
        .parent()
        .context("Missing package")?
        .canonicalize()?;
    let path = package.join(relative_path).canonicalize()?;
    ensure!(
        path.starts_with(&package),
        "External blanket manifest escapes package"
    );
    let m: Manifest = serde_json::from_slice(&fs::read(path)?)?;
    validate_manifest(&m)?;
    let configured: BTreeSet<_> = tool.proofs.iter().map(|p| p.api.clone()).collect();
    ensure!(
        configured.len() == 69
            && m.keys.iter().all(|key| configured.contains(key))
            && format!(
                "{:x}",
                Sha256::digest(
                    configured
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("\n")
                        .as_bytes()
                )
            ) == "364f6c3e48ab55960023af10a40340f12d487dccdf491d72f490eb108d992f4f",
        "External blanket expects the audited 69-API union"
    );
    let mut actual_sources = BTreeSet::from(["Cargo.toml".to_owned(), "Cargo.lock".to_owned()]);
    fn collect_sources(root: &Path, dir: &Path, out: &mut BTreeSet<String>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            ensure!(
                !kind.is_symlink(),
                "Symlink in original Rust source closure"
            );
            if kind.is_dir() {
                collect_sources(root, &entry.path(), out)?;
            } else if kind.is_file() && entry.path().extension().is_some_and(|x| x == "rs") {
                out.insert(
                    entry
                        .path()
                        .strip_prefix(root)?
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
        Ok(())
    }
    collect_sources(&package, &package.join("src"), &mut actual_sources)?;
    ensure!(actual_sources == m.source_files.keys().cloned().collect(), "External blanket source closure must include every original src Rust file, manifest and lock");
    for (name, hash) in &m.source_files {
        let path = package.join(name).canonicalize()?;
        ensure!(
            path.starts_with(&package),
            "External blanket source escapes package"
        );
        ensure!(
            format!("{:x}", Sha256::digest(fs::read(path)?)) == *hash,
            "External blanket source digest mismatch: {name}"
        );
    }
    // This establishes only metadata scope and immutable file checks. It is not
    // sufficient to create an observation: compiler regeneration is mandatory.
    Ok(m)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Manifest {
        Manifest {
            schema: 1,
            kind: "external_blanket_specialization".into(),
            local_declaration: false,
            crate_name: "rand_xoshiro".into(),
            version: "0.8.1".into(),
            target: "x86_64-unknown-linux-gnu".into(),
            compiler: "nightly-2026-09-17".into(),
            dependency: Dependency {
                name: "rand_core".into(),
                version: "0.10.1".into(),
                checksum: CORE_CHECKSUM.into(),
            },
            configurations: BTreeMap::from([
                ("default".into(), vec![]),
                ("serde".into(), vec!["serde".into()]),
            ]),
            keys: expected_keys().into_iter().collect(),
            source_files: BTreeMap::from([("Cargo.toml".into(), "a".repeat(64))]),
        }
    }
    #[test]
    fn exact_scope_and_keys() {
        assert_eq!(expected_keys().len(), 25);
        assert!(validate_manifest(&fixture()).is_ok());
    }
    #[test]
    fn reject_scope_dependency_features_and_erasure() {
        let mut m = fixture();
        m.local_declaration = true;
        assert!(validate_manifest(&m).is_err());
        let mut m = fixture();
        m.kind = "trait_default".into();
        assert!(validate_manifest(&m).is_err());
        let mut m = fixture();
        m.target = "i686-unknown-linux-gnu".into();
        assert!(validate_manifest(&m).is_err());
        let mut m = fixture();
        m.compiler = "nightly".into();
        assert!(validate_manifest(&m).is_err());
        let mut m = fixture();
        m.dependency.version = "0.10.0".into();
        assert!(validate_manifest(&m).is_err());
        let mut m = fixture();
        m.keys[1] = m.keys[0].clone();
        assert!(validate_manifest(&m).is_err());
        let mut m = fixture();
        m.keys
            .push("<rand_xoshiro::Xoshiro128Plus as rand_core::Rng>::next_u64".into());
        assert!(validate_manifest(&m).is_err());
        let mut m = fixture();
        m.version = "0.8.2".into();
        assert!(validate_manifest(&m).is_err());
        let mut m = fixture();
        m.dependency.checksum = "0".repeat(64);
        assert!(validate_manifest(&m).is_err());
        let mut m = fixture();
        m.configurations.remove("serde");
        assert!(validate_manifest(&m).is_err());
        let mut m = fixture();
        m.keys
            .retain(|key| key != "<rand_xoshiro::SplitMix64 as rand_core::Rng>::next_u32");
        m.keys.push("SplitMix64::next_u32".into());
        assert!(validate_manifest(&m).is_err());
        let mut m = fixture();
        m.source_files
            .insert("../Cargo.toml".into(), "a".repeat(64));
        assert!(validate_manifest(&m).is_err());
    }
}

fn digest(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}
fn run(command: &mut Command) -> Result<Vec<u8>> {
    let output = command
        .output()
        .context("Run external blanket compiler capture")?;
    ensure!(
        output.status.success(),
        "External blanket capture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output.stdout)
}
fn fixed_scope() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "external_blanket_scope.json"
    ))?)
}
fn check_file_map(root: &Path, files: &Value) -> Result<()> {
    for (name, expected) in files.as_object().context("Invalid fixed source closure")? {
        ensure!(relative(Path::new(name)), "Invalid fixed source filename");
        let mut cursor = root.to_owned();
        for component in Path::new(name).components() {
            cursor.push(component.as_os_str());
            ensure!(
                !fs::symlink_metadata(&cursor)?.file_type().is_symlink(),
                "Original source member/ancestor is a symlink"
            );
        }
        ensure!(
            !fs::symlink_metadata(root.join(name))?
                .file_type()
                .is_symlink(),
            "Original source member is a symlink"
        );
        let path = root.join(name).canonicalize()?;
        ensure!(
            path.starts_with(root.canonicalize()?),
            "Source symlink escapes closure"
        );
        ensure!(
            Some(digest(&path)?.as_str()) == expected.as_str(),
            "Original source/archive differs: {name}"
        );
    }
    Ok(())
}
/// Only compiler-generated and semantically validated results may reach discover.
/// This routine is shared by record and publication through aeneas::discover.
pub(crate) fn compiler_observations(project: &Project, tool: &Tool) -> Result<Vec<String>> {
    let _manifest = validate_scope(project, tool)?;
    ensure!(
        project
            .features
            .is_subset(&BTreeSet::from(["serde".to_owned()])),
        "Unproved target features selected"
    );
    let fixed = fixed_scope()?;
    let package = project
        .manifest
        .parent()
        .context("Missing package")?
        .canonicalize()?;
    ensure!(
        project.root_source.canonicalize()? == package.join("src/lib.rs").canonicalize()?,
        "External blanket original library root mismatch"
    );
    check_file_map(
        &package,
        &fixed["original_archives"]["upstream.crate"]["files"],
    )?;
    let rustc = String::from_utf8(run(Command::new("rustc").arg("-vV"))?)?;
    ensure!(
        rustc
            .lines()
            .any(|s| s == "commit-hash: 923c95cdf5ba65cea505aa2ea829f578e1506ed8")
            && rustc.lines().any(|s| s == "host: x86_64-unknown-linux-gnu"),
        "Unsupported Rust compiler identity"
    );
    let host_cfg: BTreeSet<String> =
        String::from_utf8(run(Command::new("rustc").args(["--print", "cfg"]))?)?
            .lines()
            .map(String::from)
            .collect();
    ensure!(
        host_cfg == project.cfg,
        "External blanket target cfg differs from captured target"
    );
    let charon = String::from_utf8(run(Command::new("charon").arg("version"))?)?;
    ensure!(
        charon.trim() == "0.1.291 (08d7c9d50144bd84848212c24701253585e90d1f)",
        "Unsupported Charon revision"
    );
    let aeneas = String::from_utf8(run(Command::new("aeneas").arg("-version"))?)?;
    ensure!(
        aeneas.trim() == "aeneas be86ab0",
        "Unsupported Aeneas revision"
    );
    ensure!(
        tool.version == "be86ab0f1989d04944292b345b24c435be3ad10a",
        "Wrong Aeneas proof revision"
    );
    let scratch = tempfile::tempdir().context("Create isolated compiler capture")?;
    let original = scratch.path().join("original");
    fs::create_dir(&original)?;
    // Copy only authenticated original archive members, never artifact scripts,
    // .cargo config, build caches, author rustdoc, or author MIR dumps.
    for name in fixed["original_archives"]["upstream.crate"]["files"]
        .as_object()
        .unwrap()
        .keys()
    {
        let dest = original.join(name);
        fs::create_dir_all(dest.parent().unwrap())?;
        fs::copy(package.join(name), dest)?;
    }
    let selector = scratch.path().join("selector");
    fs::create_dir_all(selector.join("src"))?;
    fs::write(
        selector.join("Cargo.toml"),
        r#"[package]
name = "xoshiro-rng-api-roots"
version = "0.1.0"
edition = "2024"
[dependencies]
rand_xoshiro = { path = "../original" }
[features]
serde = ["rand_xoshiro/serde"]
"#,
    )?;
    let lock = package.join("evidence/rng-selection-wrapper/Cargo.lock");
    ensure!(
        Some(digest(&lock)?.as_str()) == fixed["selector_cargo_lock_sha256"].as_str(),
        "Selector lock mismatch"
    );
    fs::copy(lock, selector.join("Cargo.lock"))?;
    let proof_directory = crate::aeneas::directory(project, tool)?;
    for name in [
        "RngProofs.lean",
        "XoshiroRngApiRoots/Types.lean",
        "XoshiroRngApiRoots/Funs.lean",
    ] {
        ensure!(
            proof_directory
                .join(name)
                .canonicalize()?
                .starts_with(&proof_directory),
            "Specialization proof closure escapes package"
        );
    }
    ensure!(
        Some(digest(&proof_directory.join("RngProofs.lean"))?.as_str())
            == fixed["proof_module_sha256"].as_str(),
        "External specialization proof source changed"
    );
    for (name, hash) in fixed["generated_proof_closure"].as_object().unwrap() {
        ensure!(
            Some(digest(&proof_directory.join("XoshiroRngApiRoots").join(name))?.as_str())
                == hash.as_str(),
            "Recorded specialization generated source changed"
        );
    }
    let mut source = String::from("#![no_std]\n");
    for e in fixed["bindings"]
        .as_array()
        .context("Missing fixed selectors")?
    {
        let name = e["type"].as_str().unwrap();
        let wrapper = e["wrapper"].as_str().unwrap();
        let bits = e["bits"].as_u64().unwrap();
        source.push_str(&format!("pub fn {wrapper}(s:&mut rand_xoshiro::{name})->u{bits}{{<rand_xoshiro::{name} as rand_xoshiro::rand_core::Rng>::next_u{bits}(s)}}\n"));
        let proof = tool
            .proofs
            .iter()
            .find(|p| Some(p.api.as_str()) == e["api"].as_str())
            .context("Missing fixed proof")?;
        ensure!(
            serde_json::to_value(proof)? == e["proof_binding"],
            "External specialization theorem mapping changed"
        );
    }
    ensure!(
        format!("{:x}", Sha256::digest(source.as_bytes()))
            == fixed["selector_source_sha256"].as_str().unwrap(),
        "Fixed selector construction mismatch"
    );
    fs::write(selector.join("src/lib.rs"), source)?;
    let mut meta_cmd = capture_command("cargo", &selector, &original, &selector);
    meta_cmd.args(["metadata", "--offline", "--locked", "--format-version", "1"]);
    let metadata: Value = serde_json::from_slice(&run(&mut meta_cmd)?)?;
    let core: Vec<_> = metadata["packages"]
        .as_array()
        .context("Missing resolved packages")?
        .iter()
        .filter(|p| p["name"] == "rand_core")
        .collect();
    ensure!(
        core.len() == 1
            && core[0]["version"] == "0.10.1"
            && core[0]["source"] == "registry+https://github.com/rust-lang/crates.io-index",
        "Wrong resolved dependency identity"
    );
    let core_root = Path::new(
        core[0]["manifest_path"]
            .as_str()
            .context("Missing dependency manifest")?,
    )
    .parent()
    .unwrap();
    check_file_map(
        core_root,
        &fixed["original_archives"]["rand_core-0.10.1.crate"]["files"],
    )?;
    let lock_text = fs::read_to_string(selector.join("Cargo.lock"))?;
    let lock_value: toml::Value = toml::from_str(&lock_text)?;
    let core_locks: Vec<_> = lock_value["package"]
        .as_array()
        .context("Missing lock packages")?
        .iter()
        .filter(|p| p["name"].as_str() == Some("rand_core"))
        .collect();
    ensure!(
        core_locks.len() == 1
            && core_locks[0]["version"].as_str() == Some("0.10.1")
            && core_locks[0]["checksum"].as_str() == Some(CORE_CHECKSUM),
        "Wrong dependency lock checksum"
    );
    for mode in ["default", "serde"] {
        let out = scratch.path().join(mode);
        fs::create_dir(&out)?;
        let mut command = capture_command("cargo", &original, &original, &selector);
        command
            .args(["rustdoc", "--offline", "--locked", "--manifest-path"])
            .arg(original.join("Cargo.toml"))
            .args([
                "--lib",
                "--target",
                "x86_64-unknown-linux-gnu",
                "--target-dir",
            ])
            .arg(out.join("rustdoc"));
        if mode == "serde" {
            command.args(["--features", "serde"]);
        }
        command.args([
            "--",
            "-Z",
            "unstable-options",
            "--output-format",
            "json",
            "--document-private-items",
        ]);
        run(&mut command)?;
        let mut command = capture_command("charon", &selector, &original, &selector);
        command
            .args([
                "cargo",
                "--preset",
                "aeneas",
                "--sysroot",
                "default",
                "--include",
                "rand_xoshiro",
                "--duplicate-defaulted-methods",
                "--remove-unused-self-clauses",
                "--mir",
                "optimized",
                "--rustc-arg=-Zmir-opt-level=4",
                "--rustc-arg=-Zmir-enable-passes=+Inline",
                "--rustc-arg=-Zdump-mir=Inline",
            ])
            .arg(format!(
                "--rustc-arg=-Zdump-mir-dir={}",
                out.join("mir").display()
            ))
            .arg("--dest-file")
            .arg(out.join("xoshiro_rng_api_roots.llbc"));
        if mode == "serde" {
            command.args(["--", "--features", "serde"]);
        }
        run(&mut command)?;
        let mut command = capture_command("aeneas", &selector, &original, &selector);
        command
            .args([
                "-backend",
                "lean",
                "-emit-json",
                "-split-files",
                "-abort-on-error",
                "-dest",
            ])
            .arg(out.join("generated"))
            .arg(out.join("xoshiro_rng_api_roots.llbc"));
        run(&mut command)?;
        let llbc: Value =
            serde_json::from_slice(&fs::read(out.join("xoshiro_rng_api_roots.llbc"))?)?;
        ensure!(
            llbc["has_errors"] == false
                && llbc["translated"]["target_information"][0]["key"] == "x86_64-unknown-linux-gnu",
            "Invalid selected compiler program"
        );
        let proof_dir = crate::aeneas::directory(project, tool)?.join("XoshiroRngApiRoots");
        for (name, hash) in fixed["generated_proof_closure"].as_object().unwrap() {
            let generated = out.join("generated").join(name);
            let text = fs::read_to_string(&generated)?;
            ensure!(
                !text
                    .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .any(|w| matches!(w, "axiom" | "sorry" | "opaque")),
                "Unknown external in regenerated selected program"
            );
            ensure!(
                Some(digest(&generated)?.as_str()) == hash.as_str()
                    && digest(&generated)? == digest(&proof_dir.join(name))?,
                "Regenerated selected program does not match proved body"
            );
        }
        let doc: Value = serde_json::from_slice(&fs::read(
            out.join("rustdoc/x86_64-unknown-linux-gnu/doc/rand_xoshiro.json"),
        )?)?;
        validate_document(&doc, &fixed, &out, &original, core_root)?;
    }
    check_file_map(
        &package,
        &fixed["original_archives"]["upstream.crate"]["files"],
    )?;
    check_file_map(
        core_root,
        &fixed["original_archives"]["rand_core-0.10.1.crate"]["files"],
    )?;
    eprintln!("Validated external_blanket_specialization: local_declaration=false; 25 exact public keys, 50 regenerated compiler observations (default/serde), original source and proved generated bodies unchanged");
    Ok(expected_keys().into_iter().collect())
}
fn capture_command(program: &str, cwd: &Path, original: &Path, selector: &Path) -> Command {
    let mut command = Command::new(program);
    command
        .current_dir(cwd)
        .env("CARGO_NET_OFFLINE", "true")
        .env("RUSTUP_TOOLCHAIN", "nightly-2026-09-17")
        .env("CARGO_TARGET_DIR", selector.join("target"))
        .env(
            "RUSTFLAGS",
            format!(
                "--remap-path-prefix={}=upstream --remap-path-prefix={}=selection",
                original.display(),
                selector.display()
            ),
        )
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTDOCFLAGS")
        .env_remove("RUSTC")
        .env_remove("RUSTDOC")
        .env_remove("CARGO_BUILD_RUSTC")
        .env_remove("CARGO_BUILD_RUSTC_WRAPPER")
        .env_remove("CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER");
    command
}
fn item<'a>(doc: &'a Value, id: &Value) -> Result<&'a Value> {
    doc["index"]
        .get(id.to_string())
        .context("Unresolved compiler item")
}
fn trait_path<'a>(doc: &'a Value, id: &Value) -> Result<&'a Value> {
    doc["paths"]
        .get(id.to_string())
        .context("Unresolved compiler trait path")
}
fn validate_document(
    doc: &Value,
    fixed: &Value,
    out: &Path,
    original: &Path,
    core: &Path,
) -> Result<()> {
    ensure!(doc["format_version"] == 61, "Unsupported rustdoc schema");
    let root = item(doc, &doc["root"])?;
    ensure!(
        root["name"] == "rand_xoshiro" && root["visibility"] == "public",
        "Invalid compiler crate root"
    );
    let root_items = root["inner"]["module"]["items"]
        .as_array()
        .context("Missing root items")?;
    let uses: Vec<_> = root_items
        .iter()
        .map(|id| item(doc, id))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter(|x| x["inner"].get("use").is_some() && x["visibility"] == "public")
        .collect();
    for e in fixed["bindings"].as_array().unwrap() {
        let name = e["type"].as_str().unwrap();
        let module = e["module"].as_str().unwrap();
        let bits = e["bits"].as_u64().unwrap();
        let owners: Vec<_> = uses
            .iter()
            .filter(|u| u["inner"]["use"]["name"] == name && u["inner"]["use"]["is_glob"] == false)
            .collect();
        ensure!(
            owners.len() == 1,
            "Concrete owner must have one public root reexport"
        );
        let owner = &owners[0]["inner"]["use"]["id"];
        let owner_item = item(doc, owner)?;
        let owner_path = trait_path(doc, owner)?;
        ensure!(
            owner_item["visibility"] == "public"
                && owner_item["crate_id"] == 0
                && owner_path["path"] == json!(["rand_xoshiro", module, name])
                && owner_path["kind"] == "struct",
            "Wrong concrete owner identity"
        );
        let matches = |which: &str| -> Result<Vec<&Value>> {
            let mut found = vec![];
            for value in doc["index"]
                .as_object()
                .context("Missing compiler index")?
                .values()
            {
                let Some(im) = value["inner"].get("impl") else {
                    continue;
                };
                if im["for"] != json!({"resolved_path":{"path":name,"id":owner,"args":null}})
                    || im["trait"].is_null()
                {
                    continue;
                }
                let path = trait_path(doc, &im["trait"]["id"])?;
                if path["path"] == json!(["rand_core", which]) && path["kind"] == "trait" {
                    found.push(value)
                }
            }
            Ok(found)
        };
        let external = matches("Rng")?;
        let local = matches("TryRng")?;
        ensure!(
            external.len() == 1 && local.len() == 1,
            "External/local impl selection must be unique"
        );
        let im = &external[0]["inner"]["impl"];
        let li = &local[0]["inner"]["impl"];
        let rngpath = trait_path(doc, &im["trait"]["id"])?;
        let trypath = trait_path(doc, &li["trait"]["id"])?;
        ensure!(
            rngpath["crate_id"] == trypath["crate_id"],
            "Distinct external trait crates"
        );
        let cid = rngpath["crate_id"].to_string();
        ensure!(
            doc["external_crates"][&cid]["name"] == "rand_core",
            "Wrong external trait crate"
        );
        let core_uses: Vec<_> = uses
            .iter()
            .filter(|u| u["inner"]["use"]["name"] == "rand_core")
            .collect();
        ensure!(core_uses.len() == 1, "Missing external crate reexport");
        let core_path = trait_path(doc, &core_uses[0]["inner"]["use"]["id"])?;
        ensure!(
            core_path["path"] == json!(["rand_core"])
                && core_path["crate_id"] == rngpath["crate_id"],
            "External trait reexport resolves elsewhere"
        );
        ensure!(
            im["is_negative"] == false
                && im["is_synthetic"] == false
                && im["is_unsafe"] == false
                && im["blanket_impl"] == json!({"generic":"R"})
                && im["provided_trait_methods"] == json!([]),
            "Not the actual external blanket implementation"
        );
        ensure!(
            im["trait"]["args"].is_null() && li["trait"]["args"].is_null(),
            "Unexpected external trait arguments"
        );
        ensure!(
            li["is_negative"] == false
                && li["is_synthetic"] == false
                && li["is_unsafe"] == false
                && li["blanket_impl"].is_null()
                && local[0]["crate_id"] == 0
                && li["generics"] == json!({"params":[],"where_predicates":[]}),
            "Invalid local TryRng instance"
        );
        let predicates = im["generics"]["where_predicates"]
            .as_array()
            .context("Missing blanket predicates")?;
        ensure!(predicates.len() == 1, "Unexpected blanket predicates");
        let pred = &predicates[0]["bound_predicate"];
        let bounds = pred["bounds"]
            .as_array()
            .context("Missing blanket bounds")?;
        ensure!(
            pred["type"] == json!({"generic":"R"})
                && pred["generic_params"] == json!([])
                && bounds.len() == 2,
            "Unexpected blanket generic applicability"
        );
        let mut try_bounds = 0;
        let mut sized_bounds = 0;
        for b in bounds {
            let t = &b["trait_bound"];
            let path = trait_path(doc, &t["trait"]["id"])?;
            ensure!(
                t["generic_params"] == json!([]),
                "Higher rank blanket bound unsupported"
            );
            if path["path"] == json!(["rand_core", "TryRng"]) {
                ensure!(
                    path["crate_id"] == rngpath["crate_id"]
                        && t["modifier"] == "none"
                        && t["trait"]["args"]
                            == json!({"angle_bracketed":{"args":[],"constraints":[{"name":"Error","args":null,"binding":{"equality":{"type":{"primitive":"never"}}}}]}}),
                    "Wrong TryRng Error predicate"
                );
                try_bounds += 1;
            } else {
                ensure!(
                    path["path"] == json!(["core", "marker", "Sized"])
                        && t["modifier"] == "maybe"
                        && t["trait"]["args"].is_null(),
                    "Unexpected blanket bound"
                );
                sized_bounds += 1;
            }
        }
        ensure!(
            try_bounds == 1
                && sized_bounds == 1
                && im["generics"]["params"]
                    == json!([{"name":"R","kind":{"type":{"bounds":[],"default":null,"is_synthetic":false}}}]),
            "Unexpected blanket substitution"
        );
        let errors: Vec<_> = li["items"]
            .as_array()
            .context("Missing local impl items")?
            .iter()
            .map(|id| item(doc, id))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .filter(|i| i["name"] == "Error")
            .collect();
        ensure!(
            errors.len() == 1
                && errors[0]["crate_id"] == 0
                && errors[0]["inner"]["assoc_type"]["type"] == json!({"primitive":"never"}),
            "Concrete Error is not compiler-resolved Infallible"
        );
        let local_source = fs::read_to_string(original.join("src").join(format!("{module}.rs")))?;
        ensure!(
            local_source.contains("use core::convert::Infallible;")
                && local_source.contains(&format!("impl TryRng for {name}"))
                && local_source.contains("type Error = Infallible;"),
            "Missing original Infallible source binding"
        );
        let methodname = format!("next_u{bits}");
        let methods: Vec<_> = im["items"]
            .as_array()
            .context("Missing external impl methods")?
            .iter()
            .map(|id| item(doc, id))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .filter(|i| i["name"] == methodname)
            .collect();
        ensure!(methods.len() == 1, "External method selection is ambiguous");
        let method = methods[0];
        let f = &method["inner"]["function"];
        ensure!(
            method["crate_id"] == rngpath["crate_id"]
                && f["has_body"] == true
                && f["sig"]
                    == json!({"inputs":[["self",{"borrowed_ref":{"lifetime":null,"is_mutable":true,"type":{"generic":"Self"}}}]],"output":{"primitive":format!("u{bits}")},"is_c_variadic":false}),
            "Wrong external implementation method signature"
        );
        ensure!(
            f["generics"] == json!({"params":[],"where_predicates":[]})
                && f["header"]
                    == json!({"is_const":false,"is_unsafe":false,"is_async":false,"abi":"Rust"}),
            "Unexpected method generic or calling convention"
        );
        ensure!(
            external[0]["span"]["begin"] == json!([65, 1])
                && method["span"]["begin"] == json!([if bits == 32 { 70 } else { 77 }, 5]),
            "Wrong external body source location"
        );
        for local_body in [local[0], errors[0]] {
            let path = Path::new(
                local_body["span"]["filename"]
                    .as_str()
                    .context("Missing local original body span")?,
            );
            let resolved = if path.is_absolute() {
                path.to_owned()
            } else {
                original.join(path)
            };
            ensure!(
                resolved.canonicalize()?
                    == original
                        .join("src")
                        .join(format!("{module}.rs"))
                        .canonicalize()?,
                "Local TryRng/Error body belongs to another source"
            );
        }
        for body in [external[0], method] {
            let path = Path::new(
                body["span"]["filename"]
                    .as_str()
                    .context("Missing external original source span")?,
            );
            ensure!(
                path.canonicalize()? == core.join("src/lib.rs").canonicalize()?,
                "External body span points to different source"
            );
        }
        let wrapper = e["wrapper"].as_str().unwrap();
        let mirs: Vec<_> = fs::read_dir(out.join("mir"))?
            .collect::<std::io::Result<Vec<_>>>()?
            .into_iter()
            .filter(|f| {
                let n = f.file_name().to_string_lossy().into_owned();
                n.contains(&format!(".{wrapper}.")) && n.ends_with(".Inline.after.mir")
            })
            .collect();
        ensure!(
            mirs.len() == 1,
            "Original selector Inline MIR missing/ambiguous"
        );
        let mir = fs::read_to_string(mirs[0].path())?;
        ensure!(
            mir.contains(&format!(
                "inlined <rand_xoshiro::{name} as rand_xoshiro::rand_core::Rng>::next_u{bits}"
            )) && mir.contains(&format!(
                "<rand_xoshiro::{name} as rand_xoshiro::rand_core::TryRng>::try_next_u{bits}"
            )),
            "Wrong original external/native callee chain"
        );
    }
    Ok(())
}
#[cfg(test)]
mod compiler_tests {
    use super::*;
    #[test]
    #[ignore = "Local compiler integration fixture; runs rustdoc/Charon/Aeneas twice, no network/server"]
    fn real_fifty_observations() {
        let source = std::path::PathBuf::from(std::env::var("PROOFS_RNG_SOURCE").unwrap());
        let fixed = fixed_scope().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        for name in fixed["original_archives"]["upstream.crate"]["files"]
            .as_object()
            .unwrap()
            .keys()
        {
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::copy(source.join(name), path).unwrap();
        }
        for name in [
            "evidence/rng-selection-wrapper/Cargo.lock",
            "proofs/XoshiroRngApiRoots/Types.lean",
            "proofs/XoshiroRngApiRoots/Funs.lean",
            "proofs/RngProofs.lean",
        ] {
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::copy(source.join(name), path).unwrap();
        }
        let mut config: crate::config::Config =
            toml::from_str(&fs::read_to_string(source.join("proofs.toml")).unwrap()).unwrap();
        config.tool.external_blanket_observations = Some("observation.json".into());
        let canonical: Value =
            serde_json::from_str(include_str!("external_blanket_local_keys.json")).unwrap();
        for proof in &mut config.tool.proofs {
            for row in canonical["changes"].as_array().unwrap() {
                if Some(proof.theorem.as_str()) == row["theorem"].as_str() {
                    proof.api = row["canonical"].as_str().unwrap().into();
                }
            }
        }
        let mut hashes = serde_json::Map::new();
        for (name, hash) in fixed["original_archives"]["upstream.crate"]["files"]
            .as_object()
            .unwrap()
        {
            if name.starts_with("src/") && name.ends_with(".rs")
                || name == "Cargo.toml"
                || name == "Cargo.lock"
            {
                hashes.insert(name.clone(), hash.clone());
            }
        }
        fs::write(root.join("observation.json"),serde_json::to_vec(&json!({"schema":1,"kind":"external_blanket_specialization","local_declaration":false,"crate":"rand_xoshiro","version":"0.8.1","target":"x86_64-unknown-linux-gnu","compiler":"nightly-2026-09-17","dependency":{"name":"rand_core","version":"0.10.1","checksum":CORE_CHECKSUM},"configurations":{"default":[],"serde":["serde"]},"keys":expected_keys(),"source_files":hashes})).unwrap()).unwrap();
        let project = Project {
            name: "rand_xoshiro".into(),
            version: "0.8.1".into(),
            lib_name: "rand_xoshiro".into(),
            manifest: root.join("Cargo.toml"),
            root_source: root.join("src/lib.rs"),
            features: BTreeSet::new(),
            cfg: BTreeSet::new(),
        };
        fn copy_proofs(src: &Path, dest: &Path) {
            fs::create_dir_all(dest).unwrap();
            for entry in fs::read_dir(src).unwrap() {
                let entry = entry.unwrap();
                let name = entry.file_name();
                if name == ".lake" {
                    continue;
                }
                let path = entry.path();
                if entry.file_type().unwrap().is_dir() {
                    copy_proofs(&path, &dest.join(name));
                } else if path
                    .extension()
                    .is_some_and(|ext| ext == "lean" || ext == "json")
                    || name == "lean-toolchain"
                {
                    fs::copy(path, dest.join(name)).unwrap();
                }
            }
        }
        copy_proofs(&source.join("proofs"), &root.join("proofs"));
        let mut project = project;
        project.cfg =
            String::from_utf8(run(Command::new("rustc").args(["--print", "cfg"])).unwrap())
                .unwrap()
                .lines()
                .map(String::from)
                .collect();
        let original_lib = fs::read(&project.root_source).unwrap();
        fs::write(&project.root_source, b"// wrong source").unwrap();
        assert!(compiler_observations(&project, &config.tool).is_err());
        fs::write(&project.root_source, original_lib).unwrap();
        let original_optin = config.tool.external_blanket_observations.clone();
        config.tool.external_blanket_observations = Some("../escape.json".into());
        assert!(validate_scope(&project, &config.tool).is_err());
        config.tool.external_blanket_observations = original_optin;
        for name in ["RngProofs.lean", "XoshiroRngApiRoots/Funs.lean"] {
            let path = root.join("proofs").join(name);
            let bytes = fs::read(&path).unwrap();
            fs::write(&path, b"-- substituted proof/body").unwrap();
            assert!(compiler_observations(&project, &config.tool).is_err());
            fs::write(path, bytes).unwrap();
        }
        let saved_api = config.tool.proofs[0].api.clone();
        config.tool.proofs[0].api = "wrong::API".into();
        assert!(validate_scope(&project, &config.tool).is_err());
        config.tool.proofs[0].api = saved_api;
        let observed = crate::aeneas::discover(&project, &config.tool).unwrap();
        assert_eq!(observed.len(), 69);
        for key in expected_keys() {
            assert_eq!(
                observed
                    .iter()
                    .filter(|c| c.api_paths == vec![key.clone()])
                    .count(),
                1
            );
        }
        eprintln!("69 original source callables discovered; default25+serde25 externally regenerated with exact singleton FQ keys");
        // Publication must inspect the recorded commit even after the live
        // checkout's source has changed. This is a private local Git fixture.
        fs::write(root.join("proofs.toml"), toml::to_string(&config).unwrap()).unwrap();
        run(Command::new("git").args(["init", "-q"]).current_dir(root)).unwrap();
        run(Command::new("git")
            .args(["config", "user.name", "CLI test"])
            .current_dir(root))
        .unwrap();
        run(Command::new("git")
            .args(["config", "user.email", "test@example.invalid"])
            .current_dir(root))
        .unwrap();
        let mut add = Command::new("git");
        add.current_dir(root).args(["add", "-f"]);
        for name in fixed["original_archives"]["upstream.crate"]["files"]
            .as_object()
            .unwrap()
            .keys()
        {
            add.arg(name);
        }
        add.args([
            "proofs",
            "proofs.toml",
            "observation.json",
            "evidence/rng-selection-wrapper/Cargo.lock",
        ]);
        run(&mut add).unwrap();
        run(Command::new("git")
            .args([
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-qm",
                "Private immutable source fixture",
            ])
            .current_dir(root))
        .unwrap();
        let commit = crate::git::run(root, &["rev-parse", "HEAD"]).unwrap();
        let mut recorded = crate::record::Record {
            sarif: json!({"runs":[{"versionControlProvenance":[{"revisionId":commit}]}]}),
            contracts: vec![],
        };
        fs::write(
            root.join("metadata-only.txt"),
            b"Evidence added after verification",
        )
        .unwrap();
        run(Command::new("git")
            .args(["add", "metadata-only.txt"])
            .current_dir(root))
        .unwrap();
        run(Command::new("git")
            .args([
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-qm",
                "Metadata after source record",
            ])
            .current_dir(root))
        .unwrap();
        assert_ne!(
            crate::git::run(root, &["rev-parse", "HEAD"]).unwrap(),
            commit
        );
        let original_source = fs::read(&project.root_source).unwrap();
        fs::write(
            &project.root_source,
            b"// mutable checkout must not be used",
        )
        .unwrap();
        let captured = discover_recorded(
            &project,
            &crate::ProjectArgs::default(),
            &config.tool,
            &recorded,
        )
        .unwrap();
        assert_eq!(captured.len(), 69);
        fs::write(&project.root_source, original_source).unwrap();
        let saved_precondition = config.tool.proofs[0].precondition.clone();
        config.tool.proofs[0].precondition.push_str(" changed");
        assert!(discover_recorded(
            &project,
            &crate::ProjectArgs::default(),
            &config.tool,
            &recorded
        )
        .is_err());
        config.tool.proofs[0].precondition = saved_precondition;
        fs::write(&project.root_source, b"// substituted captured source").unwrap();
        run(Command::new("git")
            .args(["add", "src/lib.rs"])
            .current_dir(root))
        .unwrap();
        run(Command::new("git")
            .args([
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-qm",
                "Bad captured source fixture",
            ])
            .current_dir(root))
        .unwrap();
        recorded.sarif["runs"][0]["versionControlProvenance"][0]["revisionId"] =
            json!(crate::git::run(root, &["rev-parse", "HEAD"]).unwrap());
        assert!(discover_recorded(
            &project,
            &crate::ProjectArgs::default(),
            &config.tool,
            &recorded
        )
        .is_err());
        assert_eq!(
            crate::git::run(root, &["worktree", "list", "--porcelain"])
                .unwrap()
                .lines()
                .filter(|line| line.starts_with("worktree "))
                .count(),
            1
        );

        let original_manifest = fs::read(&project.manifest).unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        fs::write(outside.path(), &original_manifest).unwrap();
        fs::remove_file(&project.manifest).unwrap();
        std::os::unix::fs::symlink(outside.path(), &project.manifest).unwrap();
        run(Command::new("git")
            .args(["add", "Cargo.toml"])
            .current_dir(root))
        .unwrap();
        run(Command::new("git")
            .args([
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-qm",
                "Symlink captured manifest fixture",
            ])
            .current_dir(root))
        .unwrap();
        recorded.sarif["runs"][0]["versionControlProvenance"][0]["revisionId"] =
            json!(crate::git::run(root, &["rev-parse", "HEAD"]).unwrap());
        fs::remove_file(&project.manifest).unwrap();
        fs::write(&project.manifest, original_manifest).unwrap();
        let error = discover_recorded(
            &project,
            &crate::ProjectArgs::default(),
            &config.tool,
            &recorded,
        )
        .unwrap_err();
        assert!(error.to_string().contains("symlink"), "{error:#}");
        assert_eq!(
            crate::git::run(root, &["worktree", "list", "--porcelain"])
                .unwrap()
                .lines()
                .filter(|line| line.starts_with("worktree "))
                .count(),
            1
        );
        recorded.sarif["runs"][0]["versionControlProvenance"][0]["revisionId"] =
            json!("not-a-commit");
        assert!(discover_recorded(
            &project,
            &crate::ProjectArgs::default(),
            &config.tool,
            &recorded
        )
        .is_err());
        eprintln!("Publication recorded-commit source capture succeeds after metadata-only HEAD advance and mutated live source; config/captured-source/commit/symlink substitutions rejected; worktree cleanup checked");
    }
}
#[cfg(test)]
mod semantic_tests {
    use super::*;
    #[test]
    #[ignore = "Read-only raw compiler fixture; no compilation/server"]
    fn reject_forged_compiler_chains() {
        let base = std::path::PathBuf::from(std::env::var("PROOFS_RNG_FRESH").unwrap());
        let out = base.join("default");
        let original = base.join("original");
        let doc: Value = serde_json::from_slice(
            &fs::read(out.join("rustdoc/x86_64-unknown-linux-gnu/doc/rand_xoshiro.json")).unwrap(),
        )
        .unwrap();
        let fixed = fixed_scope().unwrap();
        let index = doc["index"].as_object().unwrap();
        let (external_id, external) = index
            .iter()
            .find(|(_, v)| {
                v["inner"]["impl"]["for"]["resolved_path"]["path"] == "SplitMix64"
                    && v["inner"]["impl"]["blanket_impl"] == json!({"generic":"R"})
            })
            .unwrap();
        let im = &external["inner"]["impl"];
        let rng_id = im["trait"]["id"].to_string();
        let rng_crate = doc["paths"][&rng_id]["crate_id"].to_string();
        let (local_id, local) = index
            .iter()
            .find(|(_, v)| {
                v["inner"]["impl"]["for"] == im["for"]
                    && v["inner"]["impl"]["blanket_impl"].is_null()
                    && v["inner"]["impl"]["trait"]["path"] == "TryRng"
            })
            .unwrap();
        let error_id = local["inner"]["impl"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|id| item(&doc, id).unwrap()["name"] == "Error")
            .unwrap()
            .to_string();
        let method_id = im["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|id| item(&doc, id).unwrap()["name"] == "next_u32")
            .unwrap()
            .to_string();
        let core = Path::new(
            doc["index"][&method_id]["span"]["filename"]
                .as_str()
                .unwrap(),
        )
        .parent()
        .unwrap()
        .parent()
        .unwrap();
        validate_document(&doc, &fixed, &out, &original, core).unwrap();
        let ptr = |id: &str, tail: &str| format!("/index/{id}/{tail}");
        let tests=vec![
            ("schema", "/format_version".into(),json!(62)),
            ("traitidentity",format!("/paths/{rng_id}/path"),json!(["other_core","Rng"])),
            ("externalcrate",format!("/external_crates/{rng_crate}/name"),json!("other_core")),
            ("negative",ptr(external_id,"inner/impl/is_negative"),json!(true)),
            ("synthetic",ptr(external_id,"inner/impl/is_synthetic"),json!(true)),
            ("defaultbody",ptr(external_id,"inner/impl/provided_trait_methods"),json!(["next_u32"])),
            ("blanketsubstitution",ptr(external_id,"inner/impl/blanket_impl"),json!({"generic":"Q"})),
            ("traitargs",ptr(external_id,"inner/impl/trait/args"),json!({"angle_bracketed":{"args":[{"type":{"primitive":"u8"}}],"constraints":[]}})),
            ("extra_predicate",ptr(external_id,"inner/impl/generics/where_predicates"),json!([])),
            ("wrong_error_predicate",ptr(external_id,"inner/impl/generics/where_predicates/0/bound_predicate/bounds/0/trait_bound/trait/args/angle_bracketed/constraints/0/binding/equality/type"),json!({"primitive":"u8"})),
            ("wrong_local_error",ptr(&error_id,"inner/assoc_type/type"),json!({"primitive":"u8"})),
            ("wrong_local_owner",ptr(local_id,"inner/impl/for/resolved_path/id"),json!(999999)),
            ("return_width",ptr(&method_id,"inner/function/sig/output"),json!({"primitive":"u64"})),
            ("receiver_mutability",ptr(&method_id,"inner/function/sig/inputs/0/1/borrowed_ref/is_mutable"),json!(false)),
            ("missing_body",ptr(&method_id,"inner/function/has_body"),json!(false)),
            ("foreign_span",ptr(&method_id,"span/filename"),json!(original.join("src/lib.rs"))),
        ];
        for (name, path, value) in tests {
            let mut changed = doc.clone();
            *changed
                .pointer_mut(&path)
                .unwrap_or_else(|| panic!("missing mutation {name} {path}")) = value;
            assert!(
                validate_document(&changed, &fixed, &out, &original, core).is_err(),
                "accepted {name}"
            );
        }
        let mut duplicate = doc.clone();
        duplicate["index"]["999999"] = local.clone();
        assert!(validate_document(&duplicate, &fixed, &out, &original, core).is_err());
        let mut reexport = doc.clone();
        let rootid = doc["root"].to_string();
        let ownerid = doc["index"][&rootid]["inner"]["module"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|id| item(&doc, id).unwrap()["inner"]["use"]["name"] == "SplitMix64")
            .unwrap()
            .to_string();
        reexport["index"][&ownerid]["inner"]["use"]["id"] = json!(999999);
        assert!(validate_document(&reexport, &fixed, &out, &original, core).is_err());
        eprintln!("18 semantic mutation negatives rejected; original 25-chain document passes");
    }
}

/// Revalidate the immutable verification commit rather than a later mutable
/// checkout. The non-opt-in publication path deliberately remains unchanged.
pub(crate) fn discover_recorded(
    project: &Project,
    args: &crate::ProjectArgs,
    tool: &Tool,
    recorded: &crate::record::Record,
) -> Result<Vec<crate::scan::Contract>> {
    ensure!(
        tool.external_blanket_observations.is_some(),
        "Missing external observation opt-in"
    );
    let commit = recorded.sarif["runs"][0]["versionControlProvenance"][0]["revisionId"]
        .as_str()
        .context("Missing immutable source commit")?;
    ensure!(
        commit.len() == 40 && commit.bytes().all(|b| b.is_ascii_hexdigit()),
        "Invalid immutable source commit"
    );
    let package = project.manifest.parent().context("Missing package")?;
    let root =
        std::path::PathBuf::from(crate::git::run(package, &["rev-parse", "--show-toplevel"])?)
            .canonicalize()?;
    let relative_manifest = project
        .manifest
        .strip_prefix(&root)
        .context("Recorded package escapes source repository")?;
    let checkout = crate::git::Worktree::create(&root, commit)?;
    let checkout_root = checkout.path.canonicalize()?;
    let mut cursor = checkout_root.clone();
    for component in relative_manifest.components() {
        ensure!(
            matches!(component, Component::Normal(_)),
            "Invalid recorded package relative path"
        );
        cursor.push(component.as_os_str());
        ensure!(
            !fs::symlink_metadata(&cursor)?.file_type().is_symlink(),
            "Recorded package manifest/ancestor is a symlink"
        );
    }
    let mut selected = args.clone();
    selected.manifest_path = Some(checkout.path.join(relative_manifest));
    let captured = Project::load(&selected)?;
    ensure!(
        captured
            .manifest
            .canonicalize()?
            .starts_with(&checkout_root)
            && captured
                .root_source
                .canonicalize()?
                .starts_with(&checkout_root),
        "Captured package escapes immutable recorded worktree"
    );
    ensure!(
        captured.name == project.name && captured.version == project.version,
        "Recorded source targets a different crate/version"
    );
    let captured_tool = captured.config()?.tool;
    ensure!(
        captured_tool.name == tool.name
            && captured_tool.version == tool.version
            && crate::aeneas::configuration(&captured_tool)? == crate::aeneas::configuration(tool)?,
        "Recorded source tool/proof configuration mismatch"
    );
    crate::aeneas::discover(&captured, &captured_tool)
}
