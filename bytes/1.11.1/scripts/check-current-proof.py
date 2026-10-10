#!/usr/bin/env python3
"""Check the retained positive proof against current inputs, without old checkers.

This reuses the recorded, reviewed native/shadow correspondence for identical
source. It is not a general equivalence checker or a fresh solver execution.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys
import tomllib
import zipfile


def require(ok, message):
    if not ok:
        raise ValueError(message)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def library_config(data):
    # Tests, benches, dev dependencies and publication metadata do not participate
    # in the captured default non-test library. Features and normal deps do.
    keys = ("features", "lib", "dependencies", "build-dependencies", "target")
    config = {k: data[k] for k in keys if k in data}
    if "target" in config:
        config["target"] = {
            k: {name: value for name, value in v.items() if name != "dev-dependencies"}
            for k, v in config["target"].items()
            if any(name != "dev-dependencies" for name in v)
        }
        if not config["target"]:
            del config["target"]
    config["package"] = {
        k: data["package"][k]
        for k in ("name", "version", "edition", "build", "autolib", "autobins", "autoexamples", "links")
        if k in data["package"]
    }
    return config


def proof_statistics(proofs):
    stats = dict(files=len(proofs), prover=0, null=0, structural=0)

    def visit(node):
        if node is None:
            stats["null"] += 1
        elif isinstance(node, dict) and "children" in node:
            require(isinstance(node["children"], list), "invalid proof children")
            if not node["children"]:
                stats["structural"] += 1
            for child in node["children"]:
                visit(child)
        elif isinstance(node, dict) and isinstance(node.get("prover"), str):
            stats["prover"] += 1
        else:
            raise ValueError("unknown proof node")

    for proof in proofs:
        goals = proof.get("proofs", {}).get("Coma")
        require(isinstance(goals, dict) and goals, "empty proof")
        for node in goals.values():
            visit(node)
    require(stats["null"] == stats["structural"] == 0, "unproved or empty goals")
    return stats


def check(crate, fresh=False):
    results = crate / "verification/results"
    metadata = json.loads((results / "current.json").read_bytes())
    require(metadata["schema"] == 1 and metadata["full_original_admitted"] is False,
            "unsupported scope")
    for name, expected in metadata["source_sha256"].items():
        path = crate / name
        require(path.is_file() and digest(path.read_bytes()) == expected,
                "proof/source input changed: " + name)
    config = library_config(tomllib.loads((crate / "Cargo.toml").read_text()))
    require(config == metadata["production_lib_config"], "production library configuration changed")
    archive = results / "current.zip"
    require(digest(archive.read_bytes()) == metadata["archive_sha256"], "proof archive changed")
    with zipfile.ZipFile(archive) as package:
        names = package.namelist()
        require(len(names) == len(set(names)), "duplicate archive entries")
        files = json.loads(package.read("files.json"))

        def read(name):
            expected = files[name]
            raw = package.read("objects/" + expected)
            require(digest(raw) == expected, "archived input changed: " + name)
            return raw

        # Validate all retained current inputs, including the exact private Std
        # assumptions. No archive members are extracted or executed here.
        for name in files:
            read(name)
        expected_sources = {}
        for name, expected in files.items():
            if name.startswith("source/"):
                expected_sources[name[len("source/"):]] = expected
            elif name.startswith("production/src/"):
                expected_sources[name[len("production/"):]] = expected
        for name in ("build.rs", "extract_public.py", "Cargo.toml", "Cargo.lock", "why3find.json"):
            expected_sources[metadata["probe"] + "/" + name] = files["inputs/" + name]
        require(metadata["source_sha256"] == expected_sources, "live source inventory differs from archived inputs")
        original_config = library_config(tomllib.loads(read("production/Cargo.toml").decode()))
        require(config == original_config, "library configuration differs from captured production")
        receipt = json.loads(read("record/evidence/AZ_FULL_PROOF_RUN.json"))
        require(receipt["prover_process_exit"] == 0 and receipt["features"] == []
                and receipt["excluded"] == {}, "incomplete proof run")
        require(receipt["policy_diagnostic"] is True and receipt["policy_correspondence_exit"] == 2,
                "original diagnostic disposition was rewritten")
        correspondence = json.loads(read("record/generated/correspondence.json"))
        require(correspondence["status"] == "pass" and
                correspondence["Cargo_snapshot"]["status"] == "pass" and
                correspondence["full_original_admitted"] is False,
                "missing successful, separately recorded correspondence")
        proofs = []
        expected_coma = set()
        probe = crate / metadata["probe"]
        for row in receipt["proof_file_identity"]:
            raw = read(row["path"])
            require((digest(raw), len(raw)) == (row["sha256"], row["size"]), "proof differs from origin")
            if row["path"].endswith(".coma"):
                expected_coma.add(row["path"])
                if fresh:
                    require((probe / row["path"]).read_bytes() == raw,
                            "fresh translation differs; review before replacing retained proof")
            else:
                require(row["path"].endswith("/proof.json"), "unexpected proof output")
                if fresh:
                    raw = (probe / row["path"]).read_bytes()
                proofs.append(json.loads(raw))
        stats = proof_statistics(proofs)
        require(stats["files"] == metadata["statistics"]["files"] == len(expected_coma), "target set changed")
        if fresh:
            actual = {p.relative_to(probe).as_posix() for p in (probe / "verif").rglob("*.coma")}
            require(actual == expected_coma, "fresh proof target inventory differs")
        else:
            require(stats == receipt["statistics"] == metadata["statistics"], "proof statistics differ")
    return dict(status="pass", mode="fresh-output-check" if fresh else "retained-proof-reuse",
                statistics=stats, source_inputs=len(metadata["source_sha256"]),
                correspondence="recorded review reused for identical source",
                solver_execution="not performed by this checker", full_original_admitted=False)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--crate", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--fresh", action="store_true")
    args = parser.parse_args()
    try:
        print(json.dumps(check(args.crate.resolve(), args.fresh), indent=2))
        return 0
    except (ValueError, OSError, KeyError, zipfile.BadZipFile) as error:
        print("proof check failed: " + str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
