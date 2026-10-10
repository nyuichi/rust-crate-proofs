#!/usr/bin/env python3
"""Check one Coma own-goal tree against Why3's split_vc extraction.

This is a read-only evidence checker. It validates only the named own goal;
other proof.json entries are reported as supporting keys and are not counted
as own-body leaves. One-level split_vc trees are supported. Nested tactics are
reported as unchecked and never accepted as a complete own-goal proof.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile


CREUSOT_ROOT = Path("/workspace/proof-tools/creusot-data")
WHY3_DRIVER = CREUSOT_ROOT / "_opam/share/why3/drivers/why3.drv"
CREUSOT_WHY3_LIB = CREUSOT_ROOT / "share/why3find/packages/creusot"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def default_own_key(coma: Path) -> str:
    if coma.stem.endswith("__refines"):
        return "refines"
    text = coma.read_text(encoding="utf-8")
    functions = re.findall(r"(?m)^let rec ([^\s(]+)", text)
    if functions:
        return "vc_" + functions[-1]
    goals = re.findall(r"(?m)^goal ([^\s:]+)\s*:", text)
    if len(goals) == 1:
        return goals[0]
    raise ValueError("cannot infer one own goal; pass --own-key explicitly")


def extract_own_task_count(
    coma: Path, own_key: str, wrapper: Path, split_vc: bool
) -> int:
    with tempfile.TemporaryDirectory(prefix="http-own-goal-why3-") as temp:
        output = Path(temp)
        command = [str(wrapper), "why3", "prove"]
        if split_vc:
            command.extend(["-a", "split_vc"])
        command.extend([
            "-D",
            str(WHY3_DRIVER),
            "-L",
            str(CREUSOT_WHY3_LIB),
            "-o",
            str(output),
            str(coma),
        ])
        process = subprocess.run(command, text=True, capture_output=True, check=False)
        if process.returncode != 0:
            raise RuntimeError(
                "Why3 task extraction failed:\n"
                + process.stdout
                + process.stderr
            )

        selected = 0
        for why_file in output.glob("*.why"):
            text = why_file.read_text(encoding="utf-8")
            found = re.findall(r"(?m)^goal\s+([^\s:]+)\s*:", text)
            selected += sum(goal == own_key for goal in found)
        if selected < 1:
            raise RuntimeError(
                f"Why3 extraction produced no tasks named {own_key!r}; "
                "check --own-key and the supplied Coma target"
            )
        return selected


def proof_leaf_count(node: object) -> tuple[int, bool, str]:
    """Return (recorded leaves, complete, shape) for supported proof.json nodes."""
    if isinstance(node, dict) and isinstance(node.get("prover"), str):
        return 1, True, "direct"
    if not isinstance(node, dict) or node.get("tactic") != "split_vc":
        return 0, False, "unrecognized"
    children = node.get("children")
    if not isinstance(children, list):
        return 0, False, "malformed_split_vc"
    if not children:
        return 0, False, "zero_children"
    if any(not (isinstance(child, dict) and isinstance(child.get("prover"), str))
           for child in children):
        return len(children), False, "nested_or_unresolved"
    return len(children), True, "split_vc_one_level"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("coma", type=Path)
    parser.add_argument("--proof-json", type=Path)
    parser.add_argument("--own-key")
    parser.add_argument("--source", type=Path,
                        help="optional source file whose current hash should be recorded")
    parser.add_argument("--expected-source-sha256",
                        help="fail unless --source has this exact fingerprint")
    args = parser.parse_args()

    coma = args.coma.resolve()
    if not coma.is_file():
        parser.error(f"Coma target does not exist: {coma}")
    own_key = args.own_key or default_own_key(coma)
    proof_path = (args.proof_json or coma.with_suffix("") / "proof.json").resolve()
    if not proof_path.is_file():
        parser.error(f"proof.json does not exist: {proof_path}")

    crate_root = Path(__file__).resolve().parents[2]
    wrapper = crate_root / "scripts/run-proof.sh"
    if not wrapper.is_file():
        parser.error(f"proof wrapper does not exist: {wrapper}")

    try:
        tree = json.loads(proof_path.read_text(encoding="utf-8"))
        entries = tree["proofs"]["Coma"]
        if own_key not in entries:
            raise ValueError(f"own key {own_key!r} is missing from proof.json")
        recorded_count, recorded_complete, shape = proof_leaf_count(entries[own_key])
        if shape not in {"direct", "split_vc_one_level"}:
            raise ValueError(f"own goal tree shape {shape!r} is unsupported or incomplete")
        expected_count = extract_own_task_count(
            coma, own_key, wrapper, split_vc=(shape == "split_vc_one_level")
        )
    except (KeyError, TypeError, json.JSONDecodeError, ValueError, RuntimeError) as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 2

    source_hash = None
    if args.source:
        source_hash = sha256(args.source.resolve())
        if args.expected_source_sha256 and source_hash != args.expected_source_sha256:
            print("ERROR: source fingerprint does not match expected SHA-256", file=sys.stderr)
            return 2
    elif args.expected_source_sha256:
        parser.error("--expected-source-sha256 requires --source")

    complete = recorded_complete and recorded_count == expected_count
    result = {
        "coma": str(coma),
        "coma_sha256": sha256(coma),
        "proof_json": str(proof_path),
        "proof_json_sha256": sha256(proof_path),
        "own_goal_key": own_key,
        "own_tree_shape": shape,
        "why3_extracted_task_count": expected_count,
        "proof_json_leaf_count": recorded_count,
        "own_tree_complete": complete,
        "support_key_count": len(entries) - 1,
        "source": str(args.source.resolve()) if args.source else None,
        "source_sha256": source_hash,
        "nested_arity_checked": shape != "nested_or_unresolved",
    }
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if complete else 1


if __name__ == "__main__":
    raise SystemExit(main())
