use crate::{
    api::Api,
    config::{Config, Project},
    record,
    state::{self, Pending, State},
    ProjectArgs,
};
use anyhow::{bail, ensure, Context, Result};
use fs2::FileExt;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, IsTerminal, Write},
    path::Path,
};

/// Validate official preparation/upload bindings against the exact immutable run.
/// Only recorded candidate paths are accepted, never inferred aliases.
fn validated_run_bindings(project: &Project, tool: &crate::config::Tool, record: &record::Record,
    bytes: &[u8], response: &Value, metadata: Option<&Value>, registered_tool: &str) -> Result<Vec<(String, String)>> {
    use sha2::{Digest, Sha256};
    let parsed: Value = serde_json::from_slice(bytes)?;
    ensure!(parsed == record.sarif, "Upload bytes differ from the loaded immutable SARIF");
    let run = &record.sarif["runs"][0];
    let proof = &run["properties"]["proofs"];
    let id = run["automationDetails"]["guid"].as_str().context("Missing run ID")?;
    let digest = format!("{:x}", Sha256::digest(bytes));
    ensure!(response["id"] == id && response["sha256"] == digest,
        "Uploaded binding run/hash mismatch");
    if let Some(metadata) = metadata {
        ensure!(metadata["id"] == id && metadata["sha256"] == digest
            && metadata["crate"] == project.name && metadata["version"] == project.version
            && metadata["tool_version_id"] == registered_tool
            && metadata["size"].as_u64() == Some(bytes.len() as u64), "Uploaded binding metadata mismatch");
    }
    ensure!(run["tool"]["driver"]["name"] == tool.name && run["tool"]["driver"]["version"] == tool.version,
        "Uploaded binding tool mismatch");
    ensure!(proof["crate"] == project.name && proof["version"] == project.version,
        "Uploaded binding crate/version mismatch");
    if tool.is_aeneas() {
        ensure!(proof["aeneas"] == crate::aeneas::configuration(tool)?, "Uploaded binding configuration mismatch");
        ensure!(tool.proofs.len() == record.contracts.len() && tool.proofs.iter().all(|p|
            record.contracts.iter().filter(|c| c.harness == p.theorem && c.precondition == p.precondition
                && c.api_paths.contains(&record::qualify_api_path(&project.lib_name, &p.api))).count() == 1), "Uploaded binding configured proof mismatch");
    }
    ensure!(!record.contracts.is_empty(), "No recorded contracts");
    let apis = response["apis"].as_array().context("Missing uploaded API bindings")?;
    ensure!(apis.len() == record.contracts.len(), "Uploaded binding count mismatch");
    let mut ids = BTreeSet::new();
    let mut out = Vec::new();
    for (index, (api, contract)) in apis.iter().zip(&record.contracts).enumerate() {
        let api_id = api["api_item_id"].as_str().context("Missing uploaded API ID")?;
        ensure!(api["contract_index"].as_u64() == Some(index as u64)
            && api["display_path"].as_str().is_some_and(|p| contract.api_paths.iter().any(|x| x == p))
            && matches!(api["origin"].as_str(), Some("recorded" | "rustdoc")),
            "Uploaded binding index/path/origin mismatch");
        ensure!(api_id.len() == 64 && api_id.bytes().all(|c| c.is_ascii_hexdigit())
            && ids.insert(api_id), "Duplicate or malformed uploaded API ID");
        out.push((api_id.to_owned(), api["display_path"].as_str().unwrap().to_owned()));
    }
    Ok(out)
}

fn validate_aeneas_source_bindings(crate_name: &str, discovered: &[crate::scan::Contract], record: &record::Record,
    bindings: &[(String, String)]) -> Result<()> {
    ensure!(bindings.len() == record.contracts.len(), "Source binding count mismatch");
    for ((_, key), contract) in bindings.iter().zip(&record.contracts) {
        let matches: Vec<_> = discovered.iter().filter(|c| c.api_paths.iter().any(|p| record::qualify_api_path(crate_name,p) == *key)).collect();
        ensure!(matches.len() == 1 && matches[0].harness == contract.harness
            && matches[0].precondition == contract.precondition,
            "Returned API binding must resolve to exactly one matching original source callable: {key}");
    }
    Ok(())
}

pub struct Options {
    pub dry_run: bool,
    pub run: Option<String>,
    pub report: Option<u64>,
    pub force: bool,
    pub yes: bool,
    pub resume: bool,
}
const REPORT_FIELDS: &[&str] = &[
    "crate",
    "version",
    "title",
    "explanation",
    "trusted_assumptions",
    "tool_version_id",
    "environment",
    "evidence_url",
    "limitations",
];
const CLAIM_FIELDS: &[&str] = &[
    "id",
    "api_item_id",
    "title",
    "precondition",
    "explanation",
    "trusted_assumptions",
    "evidence_url",
    "limitations",
];
fn text(value: &Value, key: &str) -> Value {
    Value::String(value[key].as_str().unwrap_or_default().into())
}
fn snapshot(report: &Value) -> Result<Value> {
    let mut value = json!({});
    for key in REPORT_FIELDS {
        value[*key] = text(report, key);
    }
    value["dependencies"] = Value::Array(report["dependencies"].as_array().into_iter().flatten().map(|dependency| json!({"crate":dependency["crate"],"report":dependency["report"],"revision":dependency["revision"]})).collect());
    value["run_ids"] = report.get("run_ids").cloned().unwrap_or_else(|| json!([]));
    value["claims"] = Value::Array(
        report["claims"]
            .as_array()
            .context("Report is missing claims")?
            .iter()
            .map(|c| {
                let mut claim = json!({});
                for key in CLAIM_FIELDS {
                    claim[*key] = text(c, key);
                }
                claim
            })
            .collect(),
    );
    Ok(value)
}
fn key(claim: &Value) -> Result<String> {
    Ok(claim["api_item_id"]
        .as_str()
        .context("Missing API ID")?
        .into())
}
fn claims_by_key(report: &Value) -> Result<BTreeMap<String, Value>> {
    let mut claims = BTreeMap::new();
    for claim in report["claims"].as_array().context("Missing claims")? {
        ensure!(
            claims.insert(key(claim)?, claim.clone()).is_none(),
            "Existing report has multiple claims for the same API; this CLI cannot update it"
        );
    }
    Ok(claims)
}
fn merge(config: &Config, mut generated: Value, existing: Option<&Value>) -> Result<Value> {
    let mut body = if let Some(existing) = existing {
        snapshot(existing)?
    } else {
        json!({})
    };
    for field in ["crate", "version", "tool_version_id", "evidence_url"] {
        body[field] = generated[field].take();
    }
    body["run_ids"] = generated["run_ids"].take();
    body["title"] = json!(config.report.title.trim());
    for (field, value) in [
        ("explanation", &config.report.explanation),
        ("trusted_assumptions", &config.report.trusted_assumptions),
        ("limitations", &config.report.limitations),
        ("environment", &config.report.environment),
    ] {
        if let Some(value) = value {
            body[field] = json!(value);
        } else if body.get(field).is_none() {
            body[field] = json!("");
        }
    }
    if let Some(dependencies) = &config.dependencies {
        body["dependencies"] = json!(dependencies.iter().map(|(name, review)| json!({"crate":name,"report":review.report,"revision":review.revision})).collect::<Vec<_>>());
    } else if body.get("dependencies").is_none() {
        body["dependencies"] = json!([]);
    }
    let old = existing.map(claims_by_key).transpose()?.unwrap_or_default();
    let mut claims = vec![];
    for claim in generated["claims"]
        .as_array()
        .context("Missing generated claims")?
    {
        let mut merged = if let Some(previous) = old.get(&key(claim)?) {
            let mut m = json!({});
            for f in CLAIM_FIELDS {
                m[*f] = text(previous, f);
            }
            m
        } else {
            json!({})
        };
        for f in ["api_item_id", "precondition", "evidence_url"] {
            merged[f] = claim[f].clone();
        }
        claims.push(merged);
    }
    body["claims"] = Value::Array(claims);
    Ok(body)
}
fn same_content(a: &Value, b: &Value) -> bool {
    fn canonical(mut value: Value) -> Value {
        if let Some(claims) = value["claims"].as_array_mut() {
            claims.sort_by_key(|c| c["api_item_id"].to_string());
        }
        if let Some(dependencies) = value["dependencies"].as_array_mut() {
            for dependency in dependencies.iter_mut() {
                *dependency = json!({"crate":dependency["crate"],"report":dependency["report"],"revision":dependency["revision"]});
            }
            dependencies.sort_by_key(|d| {
                (
                    d["crate"].to_string(),
                    d["report"].to_string(),
                    d["revision"].to_string(),
                )
            });
        }
        value
    }
    canonical(a.clone()) == canonical(b.clone())
}
fn diff(before: &Value, after: &Value) {
    let before_dependencies = before["dependencies"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let after_dependencies = after["dependencies"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    for dependency in &before_dependencies {
        if !after_dependencies.contains(dependency) {
            eprintln!("  - dependency {dependency}");
        }
    }
    for dependency in &after_dependencies {
        if !before_dependencies.contains(dependency) {
            eprintln!("  + dependency {dependency}");
        }
    }
    if before["run_ids"] != after["run_ids"] {
        eprintln!("  run_ids: {} -> {}", before["run_ids"], after["run_ids"]);
    }
    for field in REPORT_FIELDS {
        if before[*field] != after[*field] {
            eprintln!("  {field}: {} -> {}", before[*field], after[*field]);
        }
    }
    let before = claims_by_key(before).unwrap_or_default();
    let after = claims_by_key(after).unwrap_or_default();
    for (k, c) in &after {
        match before.get(k) {
            None => eprintln!("  + claim {}", k),
            Some(old) => {
                for field in CLAIM_FIELDS {
                    if old[*field] != c[*field] {
                        eprintln!("  claim {} {field}: {} -> {}", k, old[*field], c[*field]);
                    }
                }
            }
        }
    }
    for k in before.keys() {
        if !after.contains_key(k) {
            eprintln!("  - claim {}", k);
        }
    }
}
fn validate_dependencies(api: &Api, body: &Value, proof: &Value) -> Result<()> {
    let reviews = body["dependencies"]
        .as_array()
        .context("Missing dependency reviews")?;
    if reviews.is_empty() {
        return Ok(());
    }
    ensure!(
        proof["schemaVersion"] == 2,
        "Dependency reviews require a schema version 2 run; record verification again"
    );
    let snapshot = proof["dependencies"]
        .as_array()
        .context("Recorded run has no dependency snapshot; record verification again")?;
    for review in reviews {
        let name = review["crate"]
            .as_str()
            .context("Missing dependency crate")?;
        let report_id = review["report"]
            .as_u64()
            .context("Missing dependency report ID")?;
        let revision = review["revision"]
            .as_u64()
            .context("Missing dependency revision")?;
        ensure!(
            report_id > 0 && revision > 0,
            "Dependency report and revision must be positive"
        );
        let current = api.get(&format!("/api/v1/reports/{report_id}"))?;
        ensure!(
            current["withdrawn_at"].is_null(),
            "Dependency report #{report_id} is withdrawn"
        );
        let report = api.get(&format!("/api/v1/reports/{report_id}/revisions/{revision}"))?;
        ensure!(
            report["crate"] == name,
            "Dependency {name}: report #{report_id} targets a different crate"
        );
        ensure!(snapshot.iter().any(|p| p["crate"] == name && p["version"] == report["version"] && p["source"] == "registry+https://github.com/rust-lang/crates.io-index"), "Dependency {name}: report version {} is not a crates.io dependency in the recorded run", report["version"]);
    }
    Ok(())
}
fn confirm_removal(removed: &[Value], yes: bool) -> Result<()> {
    if removed.is_empty() {
        return Ok(());
    }
    eprintln!(
        "Remove {} claim(s) from the current report (history remains):",
        removed.len()
    );
    for c in removed {
        eprintln!("  {} — {}", c["id"], c["title"]);
    }
    if yes {
        return Ok(());
    }
    ensure!(
        io::stdin().is_terminal(),
        "Deletion requires confirmation; inspect --dry-run, then use --yes to approve removals"
    );
    eprint!("Continue? [y/N] ");
    io::stderr().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    ensure!(
        matches!(input.trim().to_lowercase().as_str(), "y" | "yes"),
        "Publication cancelled"
    );
    Ok(())
}
fn complete(api: &Api, state: &mut State, path: &Path) -> Result<()> {
    let pending = state
        .pending
        .as_ref()
        .context("No interrupted publication to resume")?;
    let (status, result) = api.raw("POST", &pending.path, Some(&pending.body), Some(&pending.key))
        .context("Request saved. Use publish --resume to retry the exact request; do not create a new report")?;
    if !(200..300).contains(&status) {
        if (400..500).contains(&status) && status != 408 && status != 429 {
            state.pending = None;
            state::write(path, state)?;
            bail!("Publication rejected (HTTP {status}): {} — {}. Correct the issue and run publish again.", result["error"], result["message"]);
        }
        bail!(
            "Publication failed (HTTP {status}): {}. Request saved; use publish --resume.",
            result["error"]
        );
    }
    let id = result["id"]
        .as_u64()
        .context("Missing published report ID")?;
    let revision = result["revision_no"].as_u64().unwrap_or(1);
    // Fetch the exact committed revision, not an intervening web edit.
    let report = api.get(&format!("/api/v1/reports/{id}/revisions/{revision}"))?;
    if state.report_id != Some(id) {
        state.known_claims.clear();
    }
    for claim in report["claims"]
        .as_array()
        .context("Missing published claims")?
    {
        let api = key(claim)?;
        state.known_claims.insert(api, claim.clone());
    }
    state.report_id = Some(id);
    state.revision = Some(revision);
    state.snapshot = Some(snapshot(&report)?);
    state.pending = None;
    state::write(path, state)?;
    println!(
        "Published report #{id}, revision {revision}: {}/report/{id}",
        api.server
    );
    Ok(())
}
pub fn run(server: &str, args: &ProjectArgs, options: Options) -> Result<()> {
    let project = Project::load(args)?;
    let api = Api::new(server, true)?;
    let me = api.get("/api/v1/me")?;
    let user = me["user"]["id"]
        .as_str()
        .context("Not authenticated; run login again")?;
    let manifest = project.manifest.to_string_lossy();
    let path = record::state_dir(&project)?
        .join("publication")
        .join(format!(
            "{}.json",
            state::digest(&format!(
                "{server}\n{user}\n{manifest}\n{}\n{}",
                project.name, project.version
            ))
        ));
    fs::create_dir_all(path.parent().unwrap())?;
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path.with_extension("lock"))?;
    lock.try_lock_exclusive()
        .context("Another cargo proofs publish is running for this package/server")?;
    let mut saved: State = state::read(&path)?.unwrap_or_default();
    if options.resume {
        let pending = saved
            .pending
            .as_ref()
            .context("No interrupted publication to resume")?;
        println!(
            "Retrying saved {} request with the same idempotency key.\n{}",
            pending.path,
            serde_json::to_string_pretty(&pending.body)?
        );
        return complete(&api, &mut saved, &path);
    }
    ensure!(saved.pending.is_none(), "An interrupted publication is saved. Run publish --resume before starting another publication");
    let config = project.config()?;
    let (run_dir, recorded) = record::load(&project, options.run.as_deref())?;
    let contracts = &recorded.contracts;
    let entry = &recorded.sarif["runs"][0];
    let run_id = entry["automationDetails"]["guid"]
        .as_str()
        .context("Missing run ID")?;
    let source = &entry["versionControlProvenance"][0];
    ensure!(contracts.len() <= 100, "Too many claims");
    let tool = api.tool_version(&config.tool.name, &config.tool.version)?;
    let sarif_bytes = fs::read(run_dir.join("run.sarif.json"))?;
    crate::aeneas::exact_source_binding_keys(&project, &config.tool)?;
    let discovered = if config.tool.is_aeneas() {
        Some(if config.tool.external_blanket_observations.is_some() || config.tool.ascii_vec_source_bindings.is_some() || config.tool.oorandom_generic_source_bindings.is_some() {
            crate::external_blanket::discover_recorded(&project, args, &config.tool, &recorded)?
        } else { crate::aeneas::discover(&project, &config.tool)? })
    } else { None };
    let prepared = if options.dry_run {
        println!("Recorded SARIF (validated without storage): {}", run_dir.join("run.sarif.json").display());
        api.upload("/api/v1/runs/prepare", &sarif_bytes)?
    } else {
        api.upload(&format!("/api/v1/runs/{run_id}/sarif"), &sarif_bytes)?
    };
    let metadata = if options.dry_run { None } else { Some(api.get(&format!("/api/v1/runs/{run_id}"))?) };
    let bindings = validated_run_bindings(&project, &config.tool, &recorded, &sarif_bytes, &prepared, metadata.as_ref(), &tool)?;
    if let Some(discovered) = &discovered { validate_aeneas_source_bindings(&project.lib_name,discovered, &recorded, &bindings)?; }
    let mut claims = vec![];
    let mut seen = BTreeSet::new();
    for (index, contract) in contracts.iter().enumerate() {
        let (api_id, api_path) = bindings[index].clone();
        ensure!(
            seen.insert(api_id.clone()),
            "Multiple contracts resolve to the same public API {api_path}"
        );
        let evidence = format!(
            "{}/blob/{}/{}#L{}-L{}",
            source["repositoryUri"]
                .as_str()
                .context("Missing source repository")?,
            source["revisionId"]
                .as_str()
                .context("Missing source commit")?,
            contract.file,
            contract.first_line,
            contract.last_line
        );
        println!(
            "{} → {}\n  precondition: {}\n  evidence: {}",
            contract.harness, api_path, contract.precondition, evidence
        );
        claims.push(json!({"api_item_id":api_id,"precondition":contract.precondition,"evidence_url":evidence}));
    }
    let selected = options.report.or(saved.report_id);
    let remote = selected
        .map(|id| api.get(&format!("/api/v1/reports/{id}")))
        .transpose()?;
    if let Some(report) = &remote {
        ensure!(
            report["author_id"] == user,
            "Only the report author can publish revisions"
        );
        ensure!(
            report["crate"] == project.name && report["version"] == project.version,
            "Report targets a different crate/version; publish a new report instead"
        );
        ensure!(
            report["withdrawn_at"].is_null(),
            "Withdrawn reports cannot be revised"
        );
    }
    let attached = options.report.is_some() && options.report != saved.report_id;
    let conflict = !attached
        && remote
            .as_ref()
            .is_some_and(|r| saved.revision != r["revision_no"].as_u64());
    if conflict {
        eprintln!("Server report changed since the last CLI publication:");
        if let (Some(old), Some(new)) = (&saved.snapshot, &remote) {
            diff(old, &snapshot(new)?);
        }
        if !options.force && !options.dry_run {
            bail!("Web revision conflict. Inspect publish --dry-run; use --force to publish local changes over the latest revision");
        }
    }
    let generated = json!({"crate":project.name,"version":project.version,"tool_version_id":tool,"evidence_url":format!("{}/tree/{}", source["repositoryUri"].as_str().unwrap(), source["revisionId"].as_str().unwrap()),"run_ids":[run_id],"claims":claims});
    // Unspecified editorial fields are preserved. Force changes only fields owned by this CLI/config.
    let mut baseline = remote.clone();
    if saved.report_id == selected {
        if let Some(baseline) = &mut baseline {
            let current = claims_by_key(baseline)?;
            for claim in saved.known_claims.values() {
                if !current.contains_key(&key(claim)?) {
                    baseline["claims"]
                        .as_array_mut()
                        .context("Missing claims")?
                        .push(claim.clone());
                }
            }
        }
    }
    let mut body = merge(&config, generated, baseline.as_ref())?;
    validate_dependencies(&api, &body, &entry["properties"]["proofs"])?;
    if let Some(report) = &remote {
        body["expected_revision"] = report["revision_no"].clone();
    }
    let mut validation_body = body.clone();
    if let Some(id) = selected {
        validation_body["report_id"] = json!(id);
    }
    ensure!(
        serde_json::to_vec(&validation_body)?.len() <= 131072,
        "Report exceeds the service's 128 KiB request limit"
    );
    let preview = if options.dry_run {
        body.clone()
    } else {
        api.request(
            "POST",
            "/api/v1/reports/validate",
            Some(&validation_body),
            None,
        )?
    };
    let normalized = snapshot(&preview)?;
    let removed: Vec<Value> = if let Some(report) = &remote {
        let retained: BTreeSet<_> = body["claims"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|c| c["id"].as_str())
            .collect();
        report["claims"]
            .as_array()
            .context("Missing remote claims")?
            .iter()
            .filter(|c| !c["id"].as_str().is_some_and(|id| retained.contains(id)))
            .cloned()
            .collect()
    } else {
        vec![]
    };
    println!(
        "Report: {}\n{} claims; {} removal(s).",
        config.report.title,
        body["claims"].as_array().unwrap().len(),
        removed.len()
    );
    if let Some(report) = &remote {
        diff(&snapshot(report)?, &normalized);
    }
    if options.dry_run {
        if remote.is_none() {
            diff(&json!({"dependencies":[],"claims":[]}), &normalized);
        }
        println!("{}", serde_json::to_string_pretty(&preview)?);
        if conflict {
            eprintln!("A real publication requires --force because the server revision changed.");
        }
        println!("Dry run: no report published.");
        return Ok(());
    }
    if let Some(report) = &remote {
        if same_content(&snapshot(report)?, &normalized) {
            saved.report_id = selected;
            saved.revision = report["revision_no"].as_u64();
            saved.snapshot = Some(snapshot(report)?);
            state::write(&path, &saved)?;
            println!("No changes; no new revision published.");
            return Ok(());
        }
    }
    confirm_removal(&removed, options.yes)?;

    saved.pending = Some(Pending {
        path: selected.map_or_else(
            || "/api/v1/reports".into(),
            |id| format!("/api/v1/reports/{id}/revisions"),
        ),
        body,
        key: uuid::Uuid::new_v4().to_string(),
    });
    state::write(&path, &saved)?;
    complete(&api, &mut saved, &path)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> Config {
        toml::from_str("[report]\ntitle='Local'\n[tool]\nname='kani'\nversion='0.66.0'\n").unwrap()
    }
    #[test]
    #[ignore = "Read-only original-project source discovery requires explicit local evidence paths"]
    fn recorded_source_real_project_probe() {
        let manifest=std::env::var("PROOFS_BINDING_PROJECT").unwrap();
        let p=Project::load(&ProjectArgs{manifest_path:Some(manifest.into()),..Default::default()}).unwrap();
        let config=p.config().unwrap();
        let bytes=fs::read(std::env::var("PROOFS_BINDING_SARIF").unwrap()).unwrap();
        let sarif:Value=serde_json::from_slice(&bytes).unwrap();
        let contracts=serde_json::from_value(sarif["runs"][0]["properties"]["proofs"]["contracts"].clone()).unwrap();
        let record=record::Record{sarif,contracts};
        let discovered=crate::aeneas::discover(&p,&config.tool).unwrap();
        let mut bindings:Vec<_>=record.contracts.iter().enumerate().map(|(i,c)|(format!("{i:064x}"),c.api_paths[0].clone())).collect();
        for (index,contract) in record.contracts.iter().enumerate() {
            for path in &contract.api_paths {
                bindings[index].1=path.clone();
                validate_aeneas_source_bindings(&p.lib_name,&discovered,&record,&bindings).unwrap();
            }
            bindings[index].1=contract.api_paths[0].clone();
        }
        println!("Validated {} immutable contracts and all original recorded candidate spellings against exactly one current original source callable each; no server operation",record.contracts.len());
    }
    #[test]
    #[ignore = "Read-only diagnostic requiring explicit local evidence paths"]
    fn exact_upload_real_evidence_probe() {
        let read = |name: &str| fs::read(std::env::var(name).unwrap()).unwrap();
        let bytes=read("PROOFS_BINDING_SARIF");
        let sarif:Value=serde_json::from_slice(&bytes).unwrap();
        let response:Value=serde_json::from_slice(&read("PROOFS_BINDING_RESPONSE")).unwrap();
        let metadata:Value=serde_json::from_slice(&read("PROOFS_BINDING_METADATA")).unwrap();
        let config:Config=toml::from_str(&String::from_utf8(read("PROOFS_BINDING_CONFIG")).unwrap()).unwrap();
        let contracts=serde_json::from_value(sarif["runs"][0]["properties"]["proofs"]["contracts"].clone()).unwrap();
        let record=record::Record{sarif,contracts};
        let p=Project{name:"endian-type".into(),version:"0.2.0".into(),lib_name:"endian_type".into(),
            manifest:"Cargo.toml".into(),root_source:"src/lib.rs".into(),features:BTreeSet::new(),cfg:BTreeSet::new()};
        let bindings=validated_run_bindings(&p,&config.tool,&record,&bytes,&response,Some(&metadata), metadata["tool_version_id"].as_str().unwrap()).unwrap();
        assert_eq!(bindings.len(),66);
        println!("Validated all 66 exact returned bindings against immutable local SARIF, config and server metadata");
    }
    #[test]
    fn exact_upload_bindings_reject_cross_run_config_and_aliases() {
        use sha2::{Digest, Sha256};
        let d = tempfile::tempdir().unwrap();
        let p = Project { name:"endian-type".into(), version:"0.2.0".into(), lib_name:"endian_type".into(),
            manifest:d.path().join("Cargo.toml"), root_source:d.path().join("lib.rs"),
            features:BTreeSet::new(), cfg:BTreeSet::new() };
        let mut tool = crate::config::Tool { name:"aeneas".into(), version:"a".repeat(40),
            exact_source_bindings:Some("bindings.json".into()), ..Default::default() };
        tool.proofs=(0..66).map(|i| crate::aeneas::Proof { api:format!("<endian_type::BigEndian<T> as Trait{i}>::f"),
            module:"Proofs".into(), theorem:format!("f{i}"), precondition:"true".into() }).collect();
        let contracts: Vec<_> = tool.proofs.iter().map(|p| record::Contract { api_paths:vec![p.api.clone()],
            precondition:p.precondition.clone(), file:"Proofs.lean".into(), first_line:1,last_line:2,harness:p.theorem.clone() }).collect();
        let sarif=json!({"runs":[{"automationDetails":{"guid":"run-1"},
            "tool":{"driver":{"name":tool.name,"version":tool.version}},
            "versionControlProvenance":[{"revisionId":"source-commit"}],
            "properties":{"proofs":{"crate":p.name,"version":p.version,
            "aeneas":crate::aeneas::configuration(&tool).unwrap(),"contracts":contracts}}}]});
        let bytes=serde_json::to_vec(&sarif).unwrap();
        let record=record::Record { sarif, contracts };
        let hash=format!("{:x}",Sha256::digest(&bytes));
        let response=json!({"id":"run-1","sha256":hash,"apis":record.contracts.iter().enumerate().map(|(i,c)|
            json!({"contract_index":i,"api_item_id":format!("{i:064x}"),"display_path":c.api_paths[0],"origin":"recorded"})).collect::<Vec<_>>()});
        let metadata=json!({"id":"run-1","sha256":hash,"crate":p.name,"version":p.version,
            "tool_version_id":format!("aeneas-{}",tool.version),"size":bytes.len()});
        assert_eq!(validated_run_bindings(&p,&tool,&record,&bytes,&response,Some(&metadata), metadata["tool_version_id"].as_str().unwrap()).unwrap().len(),66);
        // A transient prepare has no stored run metadata; it still must match
        // immutable bytes, registered configuration and exact indexed paths.
        assert_eq!(validated_run_bindings(&p,&tool,&record,&bytes,&response,None,
            metadata["tool_version_id"].as_str().unwrap()).unwrap().len(),66);
        let mut rustdoc=response.clone();rustdoc["apis"][0]["origin"]=json!("rustdoc");
        assert!(validated_run_bindings(&p,&tool,&record,&bytes,&rustdoc,None,
            metadata["tool_version_id"].as_str().unwrap()).is_ok());
        let source:Vec<_>=record.contracts.iter().map(|c|crate::scan::Contract{api_paths:c.api_paths.clone(),
            precondition:c.precondition.clone(),file:"Proofs.lean".into(),first_line:1,last_line:2,harness:c.harness.clone()}).collect();
        let bound=validated_run_bindings(&p,&tool,&record,&bytes,&response,None,
            metadata["tool_version_id"].as_str().unwrap()).unwrap();
        assert!(validate_aeneas_source_bindings(&p.lib_name,&source,&record,&bound).is_ok());
        let mut ambiguous=source.clone();ambiguous.push(source[0].clone());
        assert!(validate_aeneas_source_bindings(&p.lib_name,&ambiguous,&record,&bound).is_err());
        let mut wrong_harness=source.clone();wrong_harness[0].harness="other".into();
        assert!(validate_aeneas_source_bindings(&p.lib_name,&wrong_harness,&record,&bound).is_err());
        let mut erased=bound.clone();erased[0].1="<endian_type::BigEndian as Trait0>::f".into();
        assert!(validate_aeneas_source_bindings(&p.lib_name,&source,&record,&erased).is_err());

        for (field,bad) in [("id",json!("other-run")),("sha256",json!("wrong-hash")),("apis",json!([]))] {
            let mut x=response.clone();x[field]=bad;assert!(validated_run_bindings(&p,&tool,&record,&bytes,&x,Some(&metadata), metadata["tool_version_id"].as_str().unwrap()).is_err());
        }
        for (field,bad) in [("contract_index",json!(1)),("display_path",json!("erased_alias")),("origin",json!("fabricated")),("api_item_id",json!(""))] {
            let mut x=response.clone();x["apis"][0][field]=bad;assert!(validated_run_bindings(&p,&tool,&record,&bytes,&x,Some(&metadata), metadata["tool_version_id"].as_str().unwrap()).is_err());
        }
        let mut x=response.clone();x["apis"][1]["api_item_id"]=x["apis"][0]["api_item_id"].clone();
        assert!(validated_run_bindings(&p,&tool,&record,&bytes,&x,Some(&metadata), metadata["tool_version_id"].as_str().unwrap()).is_err());
        for (field,bad) in [("crate",json!("other")),("version",json!("other")),("tool_version_id",json!("other")),("size",json!(0))] {
            let mut x=metadata.clone();x[field]=bad;assert!(validated_run_bindings(&p,&tool,&record,&bytes,&response,Some(&x), metadata["tool_version_id"].as_str().unwrap()).is_err());
        }
        let mut other=tool;other.proofs[0].api="other-key".into();
        assert!(validated_run_bindings(&p,&other,&record,&bytes,&response,Some(&metadata), metadata["tool_version_id"].as_str().unwrap()).is_err());
        let mut bytes=bytes;bytes.push(b' ');assert!(validated_run_bindings(&p,&other,&record,&bytes,&response,Some(&metadata), metadata["tool_version_id"].as_str().unwrap()).is_err());
    }
    #[test]
    fn enriched_dependency_details_do_not_change_publication_content() {
        let plain =
            json!({"claims":[],"dependencies":[{"crate":"serde","report":123,"revision":2}]});
        let enriched = json!({"claims":[],"dependencies":[{"crate":"serde","report":123,"revision":2,"version":"1.0.0","withdrawn_at":null}]});
        assert!(same_content(&plain, &enriched));
        assert_eq!(
            snapshot(&enriched).unwrap()["dependencies"],
            plain["dependencies"]
        );
    }
    #[test]
    fn dependency_reviews_preserve_clear_and_update_without_new_run() {
        let old = json!({"run_ids":["run"],"dependencies":[{"crate":"serde","report":123,"revision":2}],"claims":[]});
        let generated = json!({"run_ids":["run"],"claims":[]});
        let preserved = merge(&config(), generated.clone(), Some(&old)).unwrap();
        assert_eq!(preserved["dependencies"], old["dependencies"]);
        let mut cfg = config();
        cfg.dependencies = Some(BTreeMap::new());
        let cleared = merge(&cfg, generated.clone(), Some(&old)).unwrap();
        assert_eq!(cleared["dependencies"], json!([]));
        cfg.dependencies.as_mut().unwrap().insert(
            "serde".into(),
            crate::config::DependencyReview {
                report: std::num::NonZeroU64::new(123).unwrap(),
                revision: std::num::NonZeroU64::new(3).unwrap(),
            },
        );
        let updated = merge(&cfg, generated, Some(&old)).unwrap();
        assert_eq!(updated["run_ids"], old["run_ids"]);
        assert_eq!(updated["dependencies"][0]["revision"], 3);
        assert!(!same_content(&preserved, &updated));
    }
    #[test]
    fn preserve_ids_and_web_text_but_update_evidence() {
        let remote = json!({"title":"Web", "explanation":"Web report explanation", "claims":[{"id":"permanent","api_item_id":"api","title":"Web title","explanation":"Web claim explanation","precondition":"old","evidence_url":"old"}]});
        let generated = json!({"crate":"demo","version":"1.0.0","tool_version_id":"kani","evidence_url":"shared","claims":[{"api_item_id":"api","precondition":"new","evidence_url":"new"}]});
        let body = merge(&config(), generated, Some(&remote)).unwrap();
        assert_eq!(body["title"], "Local");
        assert_eq!(body["explanation"], "Web report explanation");
        assert_eq!(body["claims"][0]["id"], "permanent");
        assert_eq!(body["claims"][0]["title"], "Web title");
        assert_eq!(body["claims"][0]["precondition"], "new");
    }
    #[test]
    fn explicit_empty_clears_and_removed_claim_is_omitted() {
        let mut cfg = config();
        cfg.report.explanation = Some("".into());
        let old = json!({"explanation":"Web", "claims":[{"id":"removed","api_item_id":"old"}]});
        let new =
            json!({"claims":[{"api_item_id":"new","precondition":"true","evidence_url":"url"}]});
        let b = merge(&cfg, new, Some(&old)).unwrap();
        assert_eq!(b["explanation"], "");
        assert!(b["claims"][0].get("id").is_none());
    }
    #[test]
    fn interrupted_request_reuses_key_and_loads_exact_revision() {
        let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
        let origin = format!("http://{}", server.server_addr());
        let job = std::thread::spawn(move || {
            for attempt in 0..2 {
                let mut request = server.recv().unwrap();
                assert_eq!(request.url(), "/api/v1/reports");
                let mut payload = String::new();
                request.as_reader().read_to_string(&mut payload).unwrap();
                let payload: Value = serde_json::from_str(&payload).unwrap();
                assert_eq!(
                    payload["dependencies"],
                    json!([{"crate":"serde","report":123,"revision":2}])
                );
                assert!(request
                    .headers()
                    .iter()
                    .any(|h| h.field.equiv("Idempotency-Key") && h.value.as_str() == "stable-key"));
                request
                    .respond(
                        tiny_http::Response::from_string(if attempt == 0 {
                            r#"{"error":"internal_error"}"#
                        } else {
                            r#"{"id":42}"#
                        })
                        .with_status_code(if attempt == 0 {
                            500
                        } else {
                            201
                        }),
                    )
                    .unwrap();
            }
            let request = server.recv().unwrap();
            assert_eq!(request.url(), "/api/v1/reports/42/revisions/1");
            request
                .respond(tiny_http::Response::from_string(
                    r#"{"title":"Published","claims":[]}"#,
                ))
                .unwrap();
        });
        let api = Api::new(&origin, false).unwrap();
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("state.json");
        let mut state = State {
            pending: Some(Pending {
                path: "/api/v1/reports".into(),
                body: json!({"title":"Published","dependencies":[{"crate":"serde","report":123,"revision":2}]}),
                key: "stable-key".into(),
            }),
            ..State::default()
        };
        state::write(&path, &state).unwrap();
        assert!(complete(&api, &mut state, &path).is_err());
        assert!(state.pending.is_some());
        complete(&api, &mut state, &path).unwrap();
        let saved: State = state::read(&path).unwrap().unwrap();
        assert_eq!(saved.report_id, Some(42));
        assert_eq!(saved.revision, Some(1));
        assert!(saved.pending.is_none());
        job.join().unwrap();
    }
    #[test]
    fn rejected_request_can_be_corrected() {
        let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
        let origin = format!("http://{}", server.server_addr());
        let job = std::thread::spawn(move || {
            server
                .recv()
                .unwrap()
                .respond(
                    tiny_http::Response::from_string(r#"{"error":"conflict"}"#)
                        .with_status_code(409),
                )
                .unwrap();
        });
        let api = Api::new(&origin, false).unwrap();
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("state.json");
        let mut state = State {
            pending: Some(Pending {
                path: "/api/v1/reports/42/revisions".into(),
                body: json!({}),
                key: "stable-key".into(),
            }),
            ..State::default()
        };
        assert!(complete(&api, &mut state, &path).is_err());
        assert!(state.pending.is_none());
        job.join().unwrap();
    }
    #[test]
    fn rejects_ambiguous_existing_claims() {
        let r = json!({"claims":[{"api_item_id":"a"},{"api_item_id":"a"}]});
        assert!(claims_by_key(&r).is_err());
    }
}
