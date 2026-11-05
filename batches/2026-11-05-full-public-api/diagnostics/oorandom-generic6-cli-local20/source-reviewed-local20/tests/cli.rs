use std::process::Command;
#[test]
fn cargo_subcommand_and_direct_help() {
    for args in [
        vec!["--help"],
        vec!["proofs", "--help"],
        vec!["proofs", "publish", "--help"],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_cargo-proofs"))
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(String::from_utf8_lossy(&out.stdout).contains("publish"));
        assert!(String::from_utf8_lossy(&out.stdout).contains("Usage: cargo proofs"));
    }
}
#[test]
fn init_uses_workspace_metadata_and_does_not_overwrite() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("member/src")).unwrap();
    std::fs::write(
        d.path().join("Cargo.toml"),
        "[workspace]\nmembers=['member']\nresolver='2'\n[workspace.package]\nversion='1.2.3'\n",
    )
    .unwrap();
    std::fs::write(
        d.path().join("member/Cargo.toml"),
        "[package]\nname='fixture'\nversion.workspace=true\nedition='2021'\n",
    )
    .unwrap();
    std::fs::write(d.path().join("member/src/lib.rs"), "pub fn f() {}\n").unwrap();
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_cargo-proofs"))
            .current_dir(d.path())
            .args([
                "proofs",
                "init",
                "-p",
                "fixture",
                "--title",
                "My report",
                "--tool-version",
                "0.66.0",
            ])
            .output()
            .unwrap()
    };
    let out = run();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let config = std::fs::read_to_string(d.path().join("member/proofs.toml")).unwrap();
    assert!(config.contains("My report"));
    assert!(config.contains("0.66.0"));
    assert!(!config.contains("[crate]"));
    assert!(!run().status.success());
    assert_eq!(
        config,
        std::fs::read_to_string(d.path().join("member/proofs.toml")).unwrap()
    );
}

#[test]
fn init_creusot_requires_explicit_target() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir(d.path().join("src")).unwrap();
    std::fs::write(
        d.path().join("Cargo.toml"),
        "[package]\nname='fixture'\nversion='1.0.0'\nedition='2021'\n",
    )
    .unwrap();
    std::fs::write(d.path().join("src/lib.rs"), "pub fn f() {}\n").unwrap();
    let run = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_cargo-proofs"))
            .current_dir(d.path())
            .args(["init", "--tool", "creusot", "--tool-version", "0.9.0"])
            .args(extra)
            .output()
            .unwrap()
    };
    let out = run(&[]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("target"));
    assert!(!d.path().join("proofs.toml").exists());
    let out = run(&["--tool-target", "annotated"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let config = std::fs::read_to_string(d.path().join("proofs.toml")).unwrap();
    assert!(config.contains("name = \"creusot\""));
    assert!(config.contains("target = \"annotated\""));
}
