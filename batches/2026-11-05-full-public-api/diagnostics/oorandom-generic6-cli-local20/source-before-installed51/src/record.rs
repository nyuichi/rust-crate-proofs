use crate::{config::Project, scan, state, ProjectArgs};
use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub api_paths: Vec<String>,
    pub precondition: String,
    pub file: String,
    pub first_line: usize,
    pub last_line: usize,
    pub harness: String,
}
pub struct Record {
    pub sarif: Value,
    pub contracts: Vec<Contract>,
}
pub fn state_dir(project: &Project) -> Result<PathBuf> {
    let base = dirs::data_local_dir()
        .context("Cannot locate local data directory")?
        .join("cargo-proofs");
    Ok(base.join(state::digest(&project.manifest.to_string_lossy())))
}
pub fn load(project: &Project, selected: Option<&str>) -> Result<(PathBuf, Record)> {
    let base = state_dir(project)?;
    let run = match selected {
        Some(s) => s.to_owned(),
        None => state::read::<String>(&base.join("latest.json"))?
            .context("No recorded run. Run cargo proofs run -- cargo kani ... first")?,
    };
    ensure!(uuid::Uuid::parse_str(&run).is_ok(), "Invalid run ID");
    let dir = base.join("runs").join(run);
    let sarif: Value =
        state::read(&dir.join("run.sarif.json"))?.context("Recorded run not found")?;
    let entry = &sarif["runs"][0];
    let proof = &entry["properties"]["proofs"];
    ensure!(
        proof["schemaVersion"] == 2 && proof["dependencies"].is_array(),
        "Recorded run lacks a version 2 dependency snapshot; record verification again"
    );
    let contracts: Vec<Contract> = serde_json::from_value(proof["contracts"].clone())?;
    ensure!(
        entry["automationDetails"]["guid"].as_str() == dir.file_name().and_then(|s| s.to_str()),
        "Recorded run ID mismatch"
    );
    ensure!(
        entry["invocations"][0]["executionSuccessful"] == true && !contracts.is_empty(),
        "This run did not produce publishable verification results"
    );
    let tool = project.config()?.tool;
    ensure!(
        entry["tool"]["driver"]["name"] == tool.name
            && entry["tool"]["driver"]["version"] == tool.version
            && proof["target"] == serde_json::to_value(tool.target)?,
        "Tool configuration changed; run verification again"
    );
    ensure!(
        proof["crate"] == project.name && proof["version"] == project.version,
        "Recorded run targets a different crate/version"
    );
    if tool.is_aeneas() {
        ensure!(
            proof["aeneas"] == crate::aeneas::configuration(&tool)?,
            "Aeneas proof mappings changed; run verification again"
        );
    }
    let record = Record { sarif, contracts };
    Ok((dir, record))
}
fn output(cmd: &mut Command) -> Result<String> {
    let out = cmd.output()?;
    ensure!(
        out.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(String::from_utf8(out.stdout)?.trim().into())
}
fn metadata(manifest: &Path) -> Result<Value> {
    Ok(serde_json::from_str(&output(
        Command::new("cargo")
            .args([
                "metadata",
                "--format-version",
                "1",
                "--no-deps",
                "--manifest-path",
            ])
            .arg(manifest),
    )?)?)
}
pub fn run(args: &ProjectArgs, mut command: Vec<String>) -> Result<()> {
    ensure!(
        args.features.is_empty()
            && !args.all_features
            && !args.no_default_features
            && args.target.is_none(),
        "Pass compilation flags after -- as part of the verifier command"
    );
    let project = Project::load(args)?;
    let config = project.config()?;
    if config.tool.is_aeneas() {
        crate::aeneas::validate_command(&command)?;
        ensure!(
            crate::config::same_path(
                &std::env::current_dir()?,
                &crate::aeneas::directory(&project, &config.tool)?
            ),
            "Run lake build from [tool].lean_dir"
        );
        command = crate::aeneas::build_command(&config.tool);
    } else {
        ensure!(
            command.len() >= 2
                && command[0] == "cargo"
                && command[1].eq_ignore_ascii_case(&config.tool.name),
            "Use cargo proofs run -- cargo {} ...",
            config.tool.name
        );
        validate_command(&command, config.tool.is_creusot())?;
    }
    let m = metadata(&project.manifest)?;
    let workspace = PathBuf::from(
        m["workspace_root"]
            .as_str()
            .context("Missing workspace root")?,
    )
    .canonicalize()?;
    // A Git root can also hold sibling path dependencies outside the Cargo workspace.
    let root = crate::git::run(&workspace, &["rev-parse", "--show-toplevel"])
        .ok()
        .map(PathBuf::from)
        .unwrap_or(workspace.clone())
        .canonicalize()?;
    let cwd = std::env::current_dir()?.canonicalize()?;
    let relative_cwd = cwd
        .strip_prefix(&root)
        .context("Run from within the source root")?;
    let relative_manifest = project.manifest.strip_prefix(&root)?.to_owned();
    if config.tool.is_aeneas() {
        crate::aeneas::pinned_version(
            &crate::aeneas::directory(&project, &config.tool)?,
            &config.tool.version,
        )?;
    } else {
        let version = output(Command::new("cargo").args([
            config.tool.name.as_str(),
            if config.tool.is_creusot() {
                "version"
            } else {
                "--version"
            },
        ]))?;
        ensure!(
            version.split_whitespace().any(|s| s == config.tool.version),
            "Installed tool differs from proofs.toml: {version}"
        );
    }
    let source_ref = crate::git::published_source(
        &root,
        config
            .git
            .as_ref()
            .and_then(|g| g.remote.as_deref())
            .unwrap_or("origin"),
    )?;
    let rid = uuid::Uuid::new_v4().to_string();
    let base = state_dir(&project)?;
    let dir = base.join("runs").join(&rid);
    fs::create_dir_all(&dir)?;
    let stage = tempfile::tempdir()?;
    let checkout = crate::git::Worktree::create(
        &root,
        source_ref["commit"].as_str().context("Missing commit")?,
    )?;
    let source = checkout.path.canonicalize()?;
    ensure!(
        source.join(&relative_manifest).exists()
            && source
                .join(workspace.strip_prefix(&root)?)
                .join("Cargo.lock")
                .exists(),
        "Snapshot is missing Cargo.toml or Cargo.lock"
    );
    let mut selected = args.clone();
    selected.manifest_path = Some(source.join(&relative_manifest));
    if !config.tool.is_aeneas() {
        apply_cargo_flags(&mut selected, &command)?;
    }
    let prefix = format!("{}/", project.name);
    for feature in &mut selected.features {
        if let Some(local) = feature.strip_prefix(&prefix) {
            *feature = local.to_owned();
        }
    }
    let mut metadata_command = Command::new("cargo");
    metadata_command
        .args([
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(source.join(&relative_manifest))
        .current_dir(&source);
    if !selected.features.is_empty() {
        metadata_command.arg("--features").arg(
            selected
                .features
                .iter()
                .map(|f| {
                    if f.contains('/') {
                        f.clone()
                    } else {
                        format!(
                            "{}/{f}",
                            selected.package.as_deref().unwrap_or(&project.name)
                        )
                    }
                })
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    if selected.all_features {
        metadata_command.arg("--all-features");
    }
    if selected.no_default_features {
        metadata_command.arg("--no-default-features");
    }
    let platform = if let Some(target) = &selected.target {
        target.clone()
    } else {
        output(Command::new("rustc").arg("-vV"))?
            .lines()
            .find_map(|line| line.strip_prefix("host: "))
            .context("Missing rustc host")?
            .to_owned()
    };
    metadata_command.arg("--filter-platform").arg(&platform);
    let resolved: Value = serde_json::from_str(&output(&mut metadata_command)?)?;
    for package in resolved["packages"]
        .as_array()
        .context("Missing packages")?
    {
        if package["source"].is_null() {
            ensure!(
                Path::new(
                    package["manifest_path"]
                        .as_str()
                        .context("Missing local package manifest")?
                )
                .canonicalize()?
                .starts_with(&source),
                "Local dependencies must be contained in the source root"
            );
        }
    }
    // Metadata unifies features across every workspace member. Cargo tree selects
    // the actual verification package, avoiding sibling-only optional dependencies.
    let mut tree_command = Command::new("cargo");
    tree_command
        .args([
            "tree",
            "--locked",
            "--prefix",
            "none",
            "--format",
            "{p}",
            "--edges",
            "normal,build,dev",
            "--manifest-path",
        ])
        .arg(source.join(&relative_manifest))
        .args(["--package", &project.name, "--target", &platform])
        .current_dir(&source)
        .env("CARGO_TERM_COLOR", "never");
    if !selected.features.is_empty() {
        tree_command
            .arg("--features")
            .arg(selected.features.join(","));
    }
    if selected.all_features {
        tree_command.arg("--all-features");
    }
    if selected.no_default_features {
        tree_command.arg("--no-default-features");
    }
    let scoped = scope_metadata(&resolved, &output(&mut tree_command)?)?;
    let dependencies = dependency_snapshot(
        &scoped,
        &project.name,
        &source.join(&relative_manifest),
        &source,
    )?;
    let mut captured = Project::load(&selected)?;
    if !config.tool.is_aeneas() {
        captured.cfg.insert(config.tool.name.clone());
    }
    ensure!(
        captured.name == project.name,
        "Command selects a different package"
    );
    let discovered = scan::discover_for_tool(&captured, &config.tool)?;
    if config.tool.is_aeneas() {
        crate::aeneas::require_clean_build(&source.join(relative_cwd))?;
    }
    ensure!(!discovered.is_empty(), "No publishable contracts found");
    let contracts: Vec<Contract> = discovered
        .iter()
        .map(|c| {
            Ok(Contract {
                api_paths: c
                    .api_paths
                    .iter()
                    .map(|p| qualify_api_path(&captured.lib_name, p))
                    .collect(),
                precondition: c.precondition.clone(),
                file: c
                    .file
                    .strip_prefix(&source)?
                    .to_string_lossy()
                    .replace('\\', "/"),
                first_line: c.first_line,
                last_line: c.last_line,
                harness: if config.tool.is_aeneas() {
                    c.harness.clone()
                } else {
                    format!("{}::{}", captured.lib_name, c.harness)
                },
            })
        })
        .collect::<Result<_>>()?;
    // Existing proof sessions may be useful inputs; only newly written proof files count as results.
    let before = proof_files(&source)?;
    let started = chrono::Utc::now();
    println!("Recording run {rid} at {}", source_ref["commit"]);
    let env: BTreeMap<String, String> = ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTDOCFLAGS"]
        .iter()
        .filter_map(|k| std::env::var(k).ok().map(|v| (k.to_string(), v)))
        .collect();
    let mut child = Command::new(&command[0])
        .args(&command[1..])
        .current_dir(source.join(relative_cwd))
        .env("CARGO_TARGET_DIR", stage.path().join("target"))
        .env("CARGO_TERM_COLOR", "never")
        .env("NO_COLOR", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Start verification tool")?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let out = std::thread::spawn(move || tee(stdout, false));
    let err = std::thread::spawn(move || tee(stderr, true));
    let status = child.wait()?;
    let stdout = out
        .join()
        .map_err(|_| anyhow::anyhow!("stdout capture failed"))??;
    let stderr = err
        .join()
        .map_err(|_| anyhow::anyhow!("stderr capture failed"))??;
    let mut stdout =
        String::from_utf8_lossy(&stdout).replace(&source.to_string_lossy().to_string(), ".");
    let mut stderr =
        String::from_utf8_lossy(&stderr).replace(&source.to_string_lossy().to_string(), ".");
    let parsed = if config.tool.is_aeneas() {
        if status.success() {
            crate::aeneas::audit(&source.join(relative_cwd), &config.tool).map(|(out, err)| {
                stdout.push_str("\n$ lake env lean --stdin (proofs.rs theorem/axiom audit)\n");
                stdout.push_str(&out);
                stderr.push_str(&err);
                (contracts.iter().map(|c| json!({"ruleId":"aeneas.lean.theorem","kind":"pass","level":"none","message":{"text":format!("Lean checked theorem {}; axioms limited to propext, Classical.choice, Quot.sound",c.harness)},"properties":{"harness":c.harness}})).collect(), contracts.clone())
            })
        } else {
            Err(anyhow::anyhow!(
                "Lake build failed; no Aeneas theorem was certified"
            ))
        }
    } else if config.tool.is_creusot() {
        creusot_results(&source, &before, &contracts)
    } else {
        kani_results(&stdout, &contracts)
    };
    let (results, verified, diagnostic) = match parsed {
        Ok((r, c)) => (r, c, None),
        Err(e) => (vec![], vec![], Some(format!("{e:#}"))),
    };
    let finished = chrono::Utc::now();
    let unchanged = checkout.unchanged();
    let successful =
        status.success() && diagnostic.is_none() && unchanged.is_ok() && !verified.is_empty();
    let mut invocation = json!({"arguments":&command[1..],"workingDirectory":{"uri":format!("{}/",path_string(relative_cwd))},"startTimeUtc":started.to_rfc3339(),"endTimeUtc":finished.to_rfc3339(),"executionSuccessful":successful,"executableLocation":{"uri":command[0]},"environmentVariables":env});
    if let Some(code) = status.code() {
        invocation["exitCode"] = json!(code);
    }
    if let Some(d) = &diagnostic {
        invocation["toolExecutionNotifications"] = json!([{"level":"error","message":{"text":d}}]);
    }
    let mut artifacts: Vec<Value> = vec![];
    invocation["stdout"] = json!({"index":artifacts.len()});
    artifacts.push(json!({"contents":{"text":stdout}}));
    invocation["stderr"] = json!({"index":artifacts.len()});
    artifacts.push(json!({"contents":{"text":stderr}}));
    let mut sarif = json!({"version":"2.1.0","$schema":"https://json.schemastore.org/sarif-2.1.0.json","runs":[{"tool":{"driver":{"name":config.tool.name,"version":config.tool.version}},"invocations":[invocation],"results":results,"artifacts":artifacts,"versionControlProvenance":[{"repositoryUri":source_ref["repository"],"revisionId":source_ref["commit"]}],"automationDetails":{"guid":rid},"properties":{"proofs":{"schemaVersion":2,"dependencies":dependencies,"crate":project.name,"version":project.version,"contracts":verified,"target":config.tool.target,"platform":format!("{} {}",std::env::consts::OS,std::env::consts::ARCH),"rustc":output(Command::new("rustc").arg("-vV").current_dir(source.join(relative_cwd))).unwrap_or_default(),"recorderVersion":env!("CARGO_PKG_VERSION")}}}]});
    if config.tool.is_aeneas() {
        sarif["runs"][0]["properties"]["proofs"]["aeneas"] =
            crate::aeneas::configuration(&config.tool)?;
    }
    let bytes = serde_json::to_vec_pretty(&sarif)?;
    fs::write(dir.join("run.sarif.json"), &bytes)?;
    ensure!(
        bytes.len() <= 8 * 1024 * 1024,
        "SARIF including logs exceeds 8 MiB"
    );
    state::write(&base.join("latest.json"), &rid)?;
    println!("Saved run {rid}: {}", dir.display());
    unchanged?;
    ensure!(
        successful,
        "Verification did not produce publishable results. SARIF including logs saved locally. {}",
        diagnostic.unwrap_or_default()
    );
    Ok(())
}
fn scope_metadata(metadata: &Value, tree: &str) -> Result<Value> {
    let packages = metadata["packages"]
        .as_array()
        .context("Missing packages")?;
    let mut ids = std::collections::BTreeSet::new();
    for line in tree.lines().filter(|line| !line.trim().is_empty()) {
        let line = line.trim().strip_suffix(" (*)").unwrap_or(line.trim());
        let line = line.strip_suffix(" (proc-macro)").unwrap_or(line);
        let mut words = line.split_whitespace();
        let name = words.next().context("Missing cargo tree package name")?;
        let version = words
            .next()
            .and_then(|v| v.strip_prefix('v'))
            .context("Invalid cargo tree package version")?;
        let mut matches: Vec<_> = packages
            .iter()
            .filter(|p| p["name"] == name && p["version"] == version)
            .collect();
        if matches.len() > 1 {
            matches.retain(|p| {
                if p["source"].is_null() {
                    p["manifest_path"]
                        .as_str()
                        .and_then(|v| Path::new(v).parent())
                        .is_some_and(|path| {
                            line.strip_prefix(&format!("{name} v{version} ("))
                                .and_then(|suffix| suffix.strip_suffix(')'))
                                .is_some_and(|tree_path| {
                                    crate::config::same_path(path, Path::new(tree_path))
                                })
                        })
                } else if p["source"] == "registry+https://github.com/rust-lang/crates.io-index" {
                    line == format!("{name} v{version}")
                } else {
                    p["source"].as_str().is_some_and(|source| {
                        let source = source
                            .strip_prefix("git+")
                            .or_else(|| source.strip_prefix("registry+"))
                            .unwrap_or(source);
                        let url = source.split('#').next().unwrap_or(source);
                        line.contains(url)
                    })
                }
            });
        }
        ensure!(
            matches.len() == 1,
            "Cannot uniquely attribute cargo tree package {line} to locked Cargo metadata"
        );
        ids.insert(
            matches[0]["id"]
                .as_str()
                .context("Missing package ID")?
                .to_owned(),
        );
    }
    let mut scoped = metadata.clone();
    scoped["packages"]
        .as_array_mut()
        .context("Missing packages")?
        .retain(|p| p["id"].as_str().is_some_and(|id| ids.contains(id)));
    Ok(scoped)
}
fn dependency_snapshot(
    metadata: &Value,
    name: &str,
    manifest: &Path,
    source_root: &Path,
) -> Result<Vec<Value>> {
    let manifest = manifest
        .canonicalize()
        .context("Resolve selected package manifest")?;
    let source_root = source_root.canonicalize().context("Resolve source root")?;
    let packages = metadata["packages"]
        .as_array()
        .context("Missing packages")?;
    let selected = packages
        .iter()
        .find(|p| {
            p["name"] == name
                && p["manifest_path"]
                    .as_str()
                    .is_some_and(|v| crate::config::same_path(Path::new(v), &manifest))
        })
        .context("Missing selected package")?;
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .context("Missing dependency resolution")?;
    let root = selected["id"].as_str().context("Missing package ID")?;
    let mut reached = std::collections::BTreeSet::new();
    let mut queue = vec![root];
    while let Some(id) = queue.pop() {
        if !reached.insert(id) {
            continue;
        }
        let node = nodes
            .iter()
            .find(|n| n["id"] == id)
            .context("Missing resolved node")?;
        for dep in node["deps"]
            .as_array()
            .context("Missing resolved dependencies")?
        {
            queue.push(dep["pkg"].as_str().context("Missing dependency ID")?);
        }
    }
    let mut snapshot: Vec<Value> = packages
        .iter()
        .filter(|p| {
            p["id"]
                .as_str()
                .is_some_and(|id| id != root && reached.contains(id))
        })
        .map(|p| {
            let source = if let Some(source) = p["source"].as_str() {
                source.to_owned()
            } else {
                let manifest = Path::new(
                    p["manifest_path"]
                        .as_str()
                        .context("Missing dependency manifest")?,
                )
                .canonicalize()
                .context("Resolve dependency manifest")?;
                let relative = manifest
                    .parent()
                    .context("Missing dependency directory")?
                    .strip_prefix(&source_root)
                    .context("Path dependency outside source root")?;
                format!("path+{}", path_string(relative))
            };
            Ok(json!({"crate":p["name"], "version":p["version"], "source":source}))
        })
        .collect::<Result<_>>()?;
    snapshot.sort_by_key(|p| {
        (
            p["crate"].to_string(),
            p["version"].to_string(),
            p["source"].to_string(),
        )
    });
    Ok(snapshot)
}
fn path_string(path: &Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    if s.is_empty() {
        ".".into()
    } else {
        s
    }
}
fn tee(mut input: impl Read, stderr: bool) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut buf = [0; 8192];
    loop {
        let n = input.read(&mut buf)?;
        if n == 0 {
            break;
        }
        if result.len() + n <= 8 * 1024 * 1024 {
            result.extend_from_slice(&buf[..n]);
        } else {
            bail!("Verification log exceeds 8 MiB");
        }
        if stderr {
            std::io::stderr().write_all(&buf[..n])?;
        } else {
            std::io::stdout().write_all(&buf[..n])?;
        }
    }
    Ok(result)
}
fn validate_command(cmd: &[String], creusot: bool) -> Result<()> {
    for arg in &cmd[2..] {
        ensure!(
            !arg.contains('\0') && !arg.starts_with('/') && !arg.contains("../"),
            "Command paths must stay within the snapshot"
        );
        let flag = arg.split('=').next().unwrap_or("");
        ensure!(
            !matches!(
                flag,
                "--target-dir"
                    | "--out-dir"
                    | "--output"
                    | "--output-format"
                    | "--quiet"
                    | "-q"
                    | "--dry-run"
                    | "--dry-run-why3find"
                    | "--only"
                    | "--no-assertion-checks"
                    | "--no-memory-safety-checks"
                    | "--no-unwinding-checks"
                    | "--cbmc-args"
                    | "--concrete-playback"
                    | "--visualize"
            ),
            "Unsupported recording option: {flag}"
        );
    }
    if !creusot {
        ensure!(
            !cmd[2..]
                .iter()
                .any(|s| s == "--jobs" || s.starts_with("--jobs=") || s.starts_with("-j")),
            "Parallel Kani output cannot be attributed reliably; omit --jobs/-j when recording"
        );
    }
    if creusot {
        ensure!(
            !cmd.iter().any(|x| matches!(
                x.as_str(),
                "--replay" | "--ide-always" | "-i" | "--ide-on-fail"
            )),
            "Record a noninteractive proof run"
        );
    }
    Ok(())
}
fn apply_cargo_flags(args: &mut ProjectArgs, cmd: &[String]) -> Result<()> {
    let mut i = 2;
    let mut package_seen = false;
    let mut target_seen = false;
    while i < cmd.len() {
        let a = &cmd[i];
        ensure!(
            a != "--",
            "Nested -- options cannot be attributed reliably when recording"
        );
        let (flag, inline) = if let Some(value) = a.strip_prefix("-F").filter(|v| !v.is_empty()) {
            ("--features", Some(value.trim_start_matches('=')))
        } else if let Some(value) = a.strip_prefix("-p").filter(|v| !v.is_empty()) {
            ("--package", Some(value.trim_start_matches('=')))
        } else {
            a.split_once('=')
                .map_or((a.as_str(), None), |(k, v)| (k, Some(v)))
        };
        ensure!(!matches!(flag, "--manifest-path" | "--workspace" | "--all" | "--exclude"), "Unsupported package selection option {flag}; select one package with cargo proofs -p NAME");
        if matches!(flag, "--features" | "-F" | "--target" | "--package" | "-p") {
            let value = if let Some(value) = inline {
                value.to_owned()
            } else {
                i += 1;
                cmd.get(i).context("Missing Cargo option value")?.clone()
            };
            ensure!(
                !value.is_empty() && !value.starts_with('-'),
                "Missing Cargo option value for {flag}"
            );
            match flag {
                "--features" | "-F" => args.features.extend(
                    value
                        .split([',', ' '])
                        .filter(|v| !v.is_empty())
                        .map(String::from),
                ),
                "--target" => {
                    ensure!(
                        !target_seen,
                        "Multiple compilation targets are unsupported when recording"
                    );
                    target_seen = true;
                    args.target = Some(value);
                }
                _ => {
                    ensure!(!package_seen, "Select exactly one verification package");
                    package_seen = true;
                    args.package = Some(value);
                }
            }
        }
        if flag == "--all-features" {
            args.all_features = true;
        }
        if flag == "--no-default-features" {
            args.no_default_features = true;
        }
        i += 1;
    }
    Ok(())
}
pub fn kani_results(output: &str, contracts: &[Contract]) -> Result<(Vec<Value>, Vec<Contract>)> {
    let mut results = vec![];
    let mut verified = vec![];
    let mut harness = String::new();
    let mut current: Option<Value> = None;
    let mut start = 0;
    for line in output.lines().map(str::trim) {
        ensure!(
            !line.starts_with("Thread "),
            "Parallel Kani output is unsupported"
        );
        if let Some(h) = line
            .strip_prefix("Checking harness ")
            .and_then(|s| s.strip_suffix("..."))
        {
            ensure!(current.is_none(), "Incomplete Kani check");
            harness = h.into();
            start = results.len();
        } else if line.starts_with("Check ") {
            if let Some(r) = current.take() {
                ensure!(r.get("kind").is_some(), "Missing Kani status");
                results.push(r);
            }
            let name = line.split_once(": ").context("Invalid Kani check")?.1;
            current = Some(
                json!({"ruleId":name,"message":{"text":name},"properties":{"harness":harness,"check":name}}),
            );
        } else if let Some(status) = line.strip_prefix("- Status: ") {
            let r = current.as_mut().context("Kani status without a check")?;
            let kind = match status {
                "SUCCESS" => "pass",
                "FAILURE" => "fail",
                "UNREACHABLE" => "notApplicable",
                "UNDETERMINED" => "open",
                "SATISFIED" | "UNSATISFIABLE" => "informational",
                _ => bail!("Unknown Kani status {status}"),
            };
            r["kind"] = json!(kind);
            r["level"] = json!(if kind == "fail" { "error" } else { "none" });
            r["properties"]["status"] = json!(status);
        } else if let Some(desc) = line.strip_prefix("- Description: ") {
            if let Some(r) = current.as_mut() {
                r["message"] = json!({"text":desc.trim_matches('"')});
            }
        } else if let Some(loc) = line.strip_prefix("- Location: ") {
            if let Some(r) = current.as_mut() {
                r["properties"]["location"] = json!(loc);
            }
        } else if let Some(outcome) = line.strip_prefix("VERIFICATION:- ") {
            if let Some(r) = current.take() {
                ensure!(r.get("kind").is_some(), "Missing Kani status");
                results.push(r);
            }
            let matches: Vec<_> = contracts
                .iter()
                .filter(|c| {
                    c.harness == harness
                        || c.harness.strip_prefix(&format!(
                            "{}::",
                            c.api_paths
                                .first()
                                .and_then(|s| s.split("::").next())
                                .unwrap_or("")
                        )) == Some(harness.as_str())
                })
                .collect();
            if outcome == "SUCCESSFUL"
                && results.len() > start
                && results[start..].iter().all(|r| {
                    matches!(
                        r["kind"].as_str(),
                        Some("pass" | "notApplicable" | "informational")
                    )
                })
                && matches.len() == 1
            {
                let c = matches[0].clone();
                for r in &mut results[start..] {
                    r["properties"]["harness"] = json!(c.harness);
                    r["relatedLocations"] = json!([{"id":0,"physicalLocation":{"artifactLocation":{"uri":c.file},"region":{"startLine":c.first_line,"endLine":c.last_line}}}]);
                }
                verified.push(c);
            }
        }
    }
    ensure!(current.is_none(), "Incomplete Kani output");
    ensure!(
        !results.is_empty(),
        "No regular Kani check results found; quiet/terse output cannot be published"
    );
    Ok((results, verified))
}
pub(crate) fn qualify_api_path(crate_name: &str, path: &str) -> String {
    // Qualified trait paths already contain the public crate-qualified type.
    if path.starts_with('<') {
        path.to_owned()
    } else {
        format!("{crate_name}::{path}")
    }
}
fn proof_files(root: &Path) -> Result<BTreeMap<PathBuf, String>> {
    let mut out = BTreeMap::new();
    for e in ignore::WalkBuilder::new(root)
        .hidden(false)
        .ignore(false)
        .git_ignore(false)
        .build()
    {
        let e = e?;
        if e.file_name() == "proof.json" && e.file_type().is_some_and(|t| t.is_file()) {
            out.insert(
                e.path().to_owned(),
                state::digest(&String::from_utf8_lossy(&fs::read(e.path())?)),
            );
        }
    }
    Ok(out)
}
fn creusot_results(
    root: &Path,
    before: &BTreeMap<PathBuf, String>,
    contracts: &[Contract],
) -> Result<(Vec<Value>, Vec<Contract>)> {
    let mut results = vec![];
    let mut verified = vec![];
    for (path, hash) in proof_files(root)? {
        if before.get(&path) == Some(&hash) {
            continue;
        }
        let relative = path
            .strip_prefix(root)?
            .to_string_lossy()
            .replace('\\', "/");
        let matching: Vec<_> = contracts
            .iter()
            .filter(|c| {
                let suffix = format!(
                    "/{}/proof.json",
                    c.harness.split("::").skip(1).collect::<Vec<_>>().join("/")
                );
                relative.ends_with(&suffix) || creusot_span_matches(root, &path, c)
            })
            .collect();
        if matching.len() != 1 {
            continue;
        }
        let c = matching[0];
        let proof: Value = serde_json::from_slice(&fs::read(&path)?)?;
        let modules = proof["proofs"]
            .as_object()
            .context("Unknown Creusot proof.json format")?;
        let mut local = vec![];
        for (module, goals) in modules {
            for (goal, tree) in goals.as_object().context("Invalid Creusot goals")? {
                let pass = proved(tree);
                local.push(json!({"ruleId":format!("{module}/{goal}"),"kind":if pass {"pass"}else{"open"},"level":"none","message":{"text":goal},"properties":{"harness":c.harness,"proof":tree,"proofFile":relative},"locations":[{"physicalLocation":{"artifactLocation":{"uri":c.file},"region":{"startLine":c.first_line,"endLine":c.last_line}}}]}));
            }
        }
        if !local.is_empty() && local.iter().all(|r| r["kind"] == "pass") {
            verified.push(c.clone());
        }
        results.extend(local);
    }
    ensure!(!results.is_empty(),"No fresh Creusot proof.json results could be mapped to source APIs. Run the prover (not compilation only); unsupported proof layouts are rejected");
    Ok((results, verified))
}
// Creusot 0.13 emits methods under impl_Type / impl_Trait_for_Type rather
// than their source paths. Use the generated declaration span, not the lossy
// directory name, to distinguish same-named methods and external trait aliases.
fn creusot_span_matches(root: &Path, proof: &Path, contract: &Contract) -> bool {
    let Some(directory) = proof.parent() else {
        return false;
    };
    let Some(method) = contract.harness.rsplit("::").next() else {
        return false;
    };
    if directory.file_name().and_then(|n| n.to_str()) != Some(method) {
        return false;
    }
    let Ok(coma) = fs::read_to_string(directory.with_extension("coma")) else {
        return false;
    };
    let Some(header) = coma
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("(* #\""))
    else {
        return false;
    };
    let Some((file, coordinates)) = header.split_once('"') else {
        return false;
    };
    let coordinates: Vec<_> = coordinates.split_whitespace().collect();
    if coordinates.len() != 5 || coordinates[4] != "*)" {
        return false;
    }
    let (Ok(first), Ok(last)) = (
        coordinates[0].parse::<usize>(),
        coordinates[2].parse::<usize>(),
    ) else {
        return false;
    };
    let source = Path::new(file);
    let source = if source.is_absolute() {
        source.to_owned()
    } else {
        root.join(source)
    };
    let (Ok(source), Ok(expected)) = (
        source.canonicalize(),
        root.join(&contract.file).canonicalize(),
    ) else {
        return false;
    };
    source == expected
        && first >= contract.first_line
        && last >= first
        && last <= contract.last_line
}
fn proved(v: &Value) -> bool {
    if v.get("prover").and_then(Value::as_str).is_some() {
        return true;
    }
    if v.get("tactic").and_then(Value::as_str).is_some() {
        return v
            .get("children")
            .and_then(Value::as_array)
            .is_some_and(|children| !children.is_empty() && children.iter().all(proved));
    }
    if let Some(o) = v.as_object() {
        return !o.is_empty()
            && o.values().all(|v| {
                v.as_array()
                    .is_some_and(|a| !a.is_empty() && a.iter().all(proved))
            });
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    fn contract() -> Contract {
        Contract {
            api_paths: vec!["demo::f".into()],
            precondition: "true".into(),
            file: "src/lib.rs".into(),
            first_line: 1,
            last_line: 3,
            harness: "demo::check_f".into(),
        }
    }
    fn output(status: &str, outcome: &str) -> String {
        format!("Checking harness demo::check_f...\nRESULTS:\nCheck 1: f.pointer.1\n - Status: {status}\n - Description: \"pointer check\"\n - Location: src/lib.rs:1:2 in function f\nVERIFICATION:- {outcome}\n")
    }
    fn empty_project_args() -> ProjectArgs {
        ProjectArgs {
            manifest_path: None,
            package: None,
            features: vec![],
            all_features: false,
            no_default_features: false,
            target: None,
        }
    }
    #[test]
    fn cargo_flags_accumulate_features_and_reject_ambiguous_selection() {
        let mut args = empty_project_args();
        let command = [
            "cargo",
            "kani",
            "-pdemo",
            "-Fa,b",
            "--features",
            "c",
            "-F=d",
            "--target=wasm32-unknown-unknown",
        ];
        apply_cargo_flags(&mut args, &command.map(String::from)).unwrap();
        assert_eq!(args.features, ["a", "b", "c", "d"]);
        assert_eq!(args.package.as_deref(), Some("demo"));
        assert_eq!(args.target.as_deref(), Some("wasm32-unknown-unknown"));
        for flags in [
            vec!["--workspace"],
            vec!["--manifest-path=other/Cargo.toml"],
            vec!["-pa", "-pb"],
            vec!["--target=a", "--target=b"],
            vec!["--"],
        ] {
            let command: Vec<_> = ["cargo", "kani"]
                .into_iter()
                .chain(flags)
                .map(String::from)
                .collect();
            assert!(apply_cargo_flags(&mut empty_project_args(), &command).is_err());
        }
    }
    #[test]
    fn cargo_workspace_snapshot_scopes_features_renames_and_target() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers=['demo','sibling','optional','platform']\nresolver='2'\n",
        )
        .unwrap();
        for (name, extra) in [
            ("demo", "[dependencies]\nrenamed={package='optional',path='../optional',optional=true}\n[features]\nextra=['dep:renamed']\n[target.'cfg(windows)'.dependencies]\nplatform={path='../platform'}\n"),
            ("sibling", "[dependencies]\ndemo={path='../demo',features=['extra']}\n"),
            ("optional", ""), ("platform", "")
        ] {
            fs::create_dir_all(root.join(name).join("src")).unwrap();
            fs::write(root.join(name).join("src/lib.rs"), "pub fn f() {}\n").unwrap();
            fs::write(root.join(name).join("Cargo.toml"), format!("[package]\nname='{name}'\nversion='1.0.0'\nedition='2021'\n{extra}")).unwrap();
        }
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let metadata: Value = serde_json::from_str(
            &super::output(
                Command::new(&cargo)
                    .args([
                        "metadata",
                        "--offline",
                        "--format-version",
                        "1",
                        "--filter-platform",
                        "x86_64-unknown-linux-gnu",
                    ])
                    .current_dir(root),
            )
            .unwrap(),
        )
        .unwrap();
        for (platform, feature, expected) in [
            ("x86_64-unknown-linux-gnu", false, vec![]),
            ("x86_64-unknown-linux-gnu", true, vec!["optional"]),
            ("x86_64-pc-windows-msvc", false, vec!["platform"]),
        ] {
            let mut command = Command::new(&cargo);
            command
                .args([
                    "tree",
                    "--offline",
                    "--locked",
                    "--prefix",
                    "none",
                    "--format",
                    "{p}",
                    "--edges",
                    "normal,build,dev",
                    "-p",
                    "demo",
                    "--target",
                    platform,
                ])
                .current_dir(root);
            if feature {
                command.args(["--features", "extra"]);
            }
            let tree = super::output(&mut command).unwrap();
            // Use metadata for the matching target; package identities stay locked.
            let matching: Value = serde_json::from_str(
                &super::output(
                    Command::new(&cargo)
                        .args([
                            "metadata",
                            "--offline",
                            "--locked",
                            "--format-version",
                            "1",
                            "--filter-platform",
                            platform,
                        ])
                        .current_dir(root),
                )
                .unwrap(),
            )
            .unwrap();
            let scoped = scope_metadata(&matching, &tree).unwrap();
            let snapshot =
                dependency_snapshot(&scoped, "demo", &root.join("demo/Cargo.toml"), root).unwrap();
            assert_eq!(
                snapshot
                    .iter()
                    .map(|p| p["crate"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                expected
            );
        }
        assert!(metadata["resolve"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["features"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f == "extra")));
    }
    #[cfg(unix)]
    #[test]
    fn dependency_paths_resolve_symlink_aliases_and_reject_external_packages() {
        use std::os::unix::fs::symlink;
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source");
        for package in [&source, &source.join("local")] {
            fs::create_dir_all(package.join("src")).unwrap();
            fs::write(package.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
        }
        fs::write(source.join("Cargo.toml"), "[package]\nname='demo'\nversion='1.0.0'\nedition='2021'\n[dependencies]\nlocal={path='local'}\n").unwrap();
        fs::write(
            source.join("local/Cargo.toml"),
            "[package]\nname='local'\nversion='1.0.0'\nedition='2021'\n",
        )
        .unwrap();
        let source = source.canonicalize().unwrap();
        let alias = directory.path().join("source alias (canonical)");
        symlink(&source, &alias).unwrap();
        let metadata = json!({"packages":[
            {"id":"root","name":"demo","version":"1.0.0","source":null,"manifest_path":source.join("Cargo.toml")},
            {"id":"local","name":"local","version":"1.0.0","source":null,"manifest_path":source.join("local/Cargo.toml")},
            {"id":"registry","name":"local","version":"1.0.0","source":"registry+https://github.com/rust-lang/crates.io-index"}],
            "resolve":{"nodes":[{"id":"root","deps":[{"pkg":"local"}]},{"id":"local","deps":[]}]}});
        let tree = format!(
            "demo v1.0.0 ({})\nlocal v1.0.0 ({}) (proc-macro)\n",
            alias.display(),
            alias.join("local").display()
        );
        let scoped = scope_metadata(&metadata, &tree).unwrap();
        assert_eq!(scoped["packages"].as_array().unwrap().len(), 2);
        let snapshot =
            dependency_snapshot(&scoped, "demo", &alias.join("Cargo.toml"), &alias).unwrap();
        assert_eq!(snapshot[0]["source"], "path+local");
        assert_eq!(
            snapshot,
            dependency_snapshot(&scoped, "demo", &source.join("Cargo.toml"), &source).unwrap()
        );
        let mut args = empty_project_args();
        args.manifest_path = Some(alias.join("Cargo.toml"));
        assert_eq!(
            Project::load(&args).unwrap().manifest,
            source.join("Cargo.toml")
        );

        let external = directory.path().join("external");
        fs::create_dir(&external).unwrap();
        fs::write(external.join("Cargo.toml"), "").unwrap();
        symlink(&external, source.join("outside")).unwrap();
        let mut escaped = scoped;
        escaped["packages"][1]["manifest_path"] = json!(source.join("outside/Cargo.toml"));
        let error =
            dependency_snapshot(&escaped, "demo", &alias.join("Cargo.toml"), &alias).unwrap_err();
        assert!(format!("{error:#}").contains("Path dependency outside source root"));
    }

    #[test]
    fn path_dependency_sources_are_relative_and_stable() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().canonicalize().unwrap();
        fs::create_dir(source.join("local")).unwrap();
        fs::write(source.join("Cargo.toml"), "").unwrap();
        fs::write(source.join("local/Cargo.toml"), "").unwrap();
        let metadata = json!({"packages":[{"id":"root","name":"demo","manifest_path":source.join("Cargo.toml")},{"id":"local","name":"local","version":"1.0.0","source":null,"manifest_path":source.join("local/Cargo.toml")}],"resolve":{"nodes":[{"id":"root","deps":[{"pkg":"local"}]},{"id":"local","deps":[]}]}});
        let snapshot =
            dependency_snapshot(&metadata, "demo", &source.join("Cargo.toml"), &source).unwrap();
        assert_eq!(snapshot[0]["source"], "path+local");
    }
    #[test]
    fn dependency_snapshot_follows_ids_including_renames_and_multiple_versions() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().canonicalize().unwrap();
        fs::write(source.join("Cargo.toml"), "").unwrap();
        let metadata = json!({"packages":[
            {"id":"root","name":"demo","manifest_path":source.join("Cargo.toml")},
            {"id":"a1","name":"a","version":"1.0.0","source":"registry+https://github.com/rust-lang/crates.io-index"},
            {"id":"a2","name":"a","version":"2.0.0","source":"git+https://example.com/a"},
            {"id":"unused","name":"unused","version":"1","source":null}],
            "resolve":{"nodes":[{"id":"root","deps":[{"name":"renamed","pkg":"a1"}]},{"id":"a1","deps":[{"pkg":"a2"}]},{"id":"a2","deps":[{"pkg":"a1"}]}]}});
        let snapshot =
            dependency_snapshot(&metadata, "demo", &source.join("Cargo.toml"), &source).unwrap();
        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot[0]["crate"], "a");
        assert_eq!(snapshot[0]["version"], "1.0.0");
        assert_eq!(snapshot[1]["source"], "git+https://example.com/a");
    }
    #[test]
    fn maps_only_observed_harnesses() {
        let c = contract();
        let (r, v) =
            kani_results(&output("SUCCESS", "SUCCESSFUL"), std::slice::from_ref(&c)).unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(r[0]["kind"], "pass");
        assert_eq!(
            r[0]["properties"]["location"],
            "src/lib.rs:1:2 in function f"
        );
        let mut other = c;
        other.harness = "demo::unexecuted".into();
        assert!(kani_results(&output("SUCCESS", "SUCCESSFUL"), &[other])
            .unwrap()
            .1
            .is_empty());
    }
    #[test]
    fn accepts_crate_relative_harness_without_suffix_guessing() {
        let output = output("SUCCESS", "SUCCESSFUL").replace(
            "Checking harness demo::check_f...",
            "Checking harness check_f...",
        );
        let (_, verified) = kani_results(&output, &[contract()]).unwrap();
        assert_eq!(verified.len(), 1);
        let nested = output.replace(
            "Checking harness check_f...",
            "Checking harness other::check_f...",
        );
        assert!(kani_results(&nested, &[contract()]).unwrap().1.is_empty());
        assert!(kani_results(&output, &[contract(), contract()])
            .unwrap()
            .1
            .is_empty());
    }
    #[test]
    fn rejects_missing_unknown_and_failed_results() {
        assert!(kani_results("VERIFICATION:- SUCCESSFUL", &[contract()]).is_err());
        assert!(kani_results(&output("NEW_STATUS", "SUCCESSFUL"), &[contract()]).is_err());
        for status in ["FAILURE", "UNDETERMINED"] {
            assert!(kani_results(&output(status, "SUCCESSFUL"), &[contract()])
                .unwrap()
                .1
                .is_empty());
        }
    }
    #[test]
    fn unreachable_is_not_labeled_pass() {
        assert_eq!(
            kani_results(&output("UNREACHABLE", "SUCCESSFUL"), &[contract()])
                .unwrap()
                .0[0]["kind"],
            "notApplicable"
        );
    }
    #[test]
    fn creusot_requires_fresh_proof_files() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("verif/demo/check_f/proof.json");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(
            &p,
            r#"{"proofs":{"M":{"vc_f":{"prover":"z3","time":0.1}}}}"#,
        )
        .unwrap();
        assert_eq!(
            creusot_results(d.path(), &BTreeMap::new(), &[contract()])
                .unwrap()
                .1
                .len(),
            1
        );
        assert!(creusot_results(d.path(), &proof_files(d.path()).unwrap(), &[contract()]).is_err());
    }

    #[test]
    fn preserves_qualified_trait_api_paths() {
        assert_eq!(qualify_api_path("demo", "S::new"), "demo::S::new");
        assert_eq!(
            qualify_api_path("demo", "<demo::S as demo::T>::f"),
            "<demo::S as demo::T>::f"
        );
    }

    #[test]
    fn native_why3find_tactic_requires_all_children_proved() {
        let pass = json!({"tactic":"split_vc","children":[{"prover":"z3","time":0.1},{"tactic":"split_vc","children":[{"prover":"cvc5","time":0.1}]}]});
        assert!(proved(&pass));
        for fail in [
            json!({"tactic":"split_vc","children":[{"prover":"z3"},{}]}),
            json!({"tactic":"split_vc","children":[]}),
            json!({"tactic":"split_vc"}),
            json!({"children":[{}]}),
        ] {
            assert!(!proved(&fail));
        }
    }

    #[test]
    fn creusot_impl_sessions_require_matching_source_span() {
        let d = tempfile::tempdir().unwrap();
        fs::create_dir_all(d.path().join("src")).unwrap();
        fs::write(d.path().join("src/lib.rs"), "// source\n").unwrap();
        let mut c = contract();
        c.harness = "demo::<S as core::hash::Hasher>::finish".into();
        let session = d
            .path()
            .join("verif/demo_rlib/impl_Hasher_for_S/finish/proof.json");
        fs::create_dir_all(session.parent().unwrap()).unwrap();
        fs::write(
            &session,
            r#"{"proofs":{"M":{"vc_finish":{"prover":"z3","time":0.1}}}}"#,
        )
        .unwrap();
        let coma = session.parent().unwrap().with_extension("coma");
        let header = format!(
            "(* #\"{}\" 2 0 2 30 *)\n",
            d.path().join("src/lib.rs").display()
        );
        fs::write(&coma, header).unwrap();
        let mut other = c.clone();
        other.harness = "demo::<S as Other>::finish".into();
        other.first_line = 10;
        other.last_line = 20;
        let (_, verified) =
            creusot_results(d.path(), &BTreeMap::new(), &[c.clone(), other]).unwrap();
        assert_eq!(verified.len(), 1);
        assert_eq!(verified[0].harness, c.harness);
        assert!(creusot_results(d.path(), &proof_files(d.path()).unwrap(), &[c.clone()]).is_err());
        fs::write(&coma, "(* #\"src/other.rs\" 2 0 2 30 *)\n").unwrap();
        assert!(creusot_results(d.path(), &BTreeMap::new(), &[c.clone()]).is_err());
        fs::write(&coma, "(* #\"src/lib.rs\" 20 0 20 30 *)\n").unwrap();
        assert!(creusot_results(d.path(), &BTreeMap::new(), &[c.clone()]).is_err());
        fs::write(&coma, "(* #\"src/lib.rs\" 2 0 2 30 *)\n").unwrap();
        c.harness = "demo::S::finish".into();
        assert_eq!(
            creusot_results(d.path(), &BTreeMap::new(), &[c.clone()])
                .unwrap()
                .1
                .len(),
            1
        );
        fs::write(&session, r#"{"proofs":{"M":{"vc_finish":{}}}}"#).unwrap();
        assert!(creusot_results(d.path(), &BTreeMap::new(), &[c])
            .unwrap()
            .1
            .is_empty());
    }
}
