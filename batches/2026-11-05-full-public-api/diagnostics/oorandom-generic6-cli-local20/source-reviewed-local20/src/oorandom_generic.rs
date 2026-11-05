//! Finite original-core conditional-family source gate, separate from xoshiro and ASCII.
//! Finite compiler-owned capture validated locally; not installed or published.
use anyhow::{ensure, Context, Result};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::{Component, Path}};
use crate::config::{Project, Tool};
#[path="oorandom_generic_capture.rs"]
mod capture;
const DATA: &str = include_str!("oorandom_generic_scope.json");
fn inside(package: &Path, relative: &Path) -> Result<std::path::PathBuf> {
    ensure!(!relative.as_os_str().is_empty() && relative.components().all(|c|matches!(c,Component::Normal(_))), "Oor generic manifest must be package-relative");
    let mut p=package.to_path_buf();
    for c in relative.components() { p.push(c); ensure!(!fs::symlink_metadata(&p)?.file_type().is_symlink(), "Oor generic manifest/ancestor symlink rejected"); }
    let p=p.canonicalize()?; ensure!(p.starts_with(package),"Oor generic manifest escapes captured package"); Ok(p)
}
fn validate_scope(project: &Project, tool: &Tool) -> Result<Value> {
    ensure!(project.name=="oorandom" && project.version=="11.1.5", "Oor generic source bindings support only oorandom11.1.5");
    ensure!(project.features.is_empty(), "Oor generic source bindings support original default configuration only");
    ensure!(tool.exact_source_bindings.is_none() && tool.external_blanket_observations.is_none() && tool.ascii_vec_source_bindings.is_none(), "Oor generic source bindings cannot combine other source gates");
    let fixed:Value=serde_json::from_str(DATA)?;
    ensure!(tool.version==fixed["tool_version"] && serde_json::to_value(&tool.proofs)?==fixed["configured_bindings20"], "Oor generic requires exact old14 plus reviewed conditional6 mappings");
    let package=project.manifest.parent().context("Missing package")?.canonicalize()?;
    let p=inside(&package,tool.oorandom_generic_source_bindings.as_deref().context("Missing Oor generic opt-in")?)?;
    let manifest:Value=serde_json::from_slice(&fs::read(p)?)?;
    let keys:BTreeSet<String>=fixed["keys"].as_array().context("Missing finite6 keys")?.iter().map(|k|k.as_str().unwrap().to_owned()).collect();
    ensure!(keys.len()==6,"Invalid finite generic6 scope");
    ensure!(manifest==serde_json::json!({"schema":1,"crate":"oorandom","version":"11.1.5","kind":"external_conditional_blanket","local_declaration":false,"fixed_scope_sha256":format!("{:x}",Sha256::digest(DATA.as_bytes())),"keys":fixed["keys"]}),"Oor generic finite manifest/source keys mismatch");
    Ok(fixed)
}
/// Both record and captured publication enter here. Six exact keys are returned
/// only after compiler-owned rustdoc/core/proof regeneration and hash checks.
pub(crate) fn compiler_observations(project:&Project,tool:&Tool)->Result<Vec<String>> {
    let fixed=validate_scope(project,tool)?;
    let work=tempfile::tempdir()?;
    let original=capture::stage_original(project,work.path(),&fixed)?;
    let (_raw_document,memberships)=capture::capture_rustdoc(&original,&work.path().join("rustdoc"),&fixed)?;
    let expected: BTreeSet<String>=fixed["keys"].as_array().unwrap().iter().map(|k|k.as_str().unwrap().to_owned()).collect();
    let actual: BTreeSet<String>=memberships.iter().map(|m|m["api"].as_str().unwrap().to_owned()).collect();
    ensure!(memberships.len()==6 && actual==expected,"Fresh original compiler memberships differ from the exact six conditional families");
    let package=project.manifest.parent().context("Missing package")?.canonicalize()?;
    capture::capture_generation(&original,work.path(),&package,&fixed)?;
    Ok(actual.into_iter().collect())
}
