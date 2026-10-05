#!/usr/bin/env python3
"""Check the exact source/CoMa shape used to prepare (not run) a ground diagnostic."""
from pathlib import Path
import hashlib
import re
import sys

fixture = Path(__file__).resolve().parent
allowed_evidence = (fixture / "evidence").resolve()
evidence = Path((fixture / "translation.latest").read_text().strip()).resolve()
if evidence.parent != allowed_evidence or not evidence.name.startswith("translation-"):
    raise SystemExit(f"SHAPE CHECK FAILED: translation path is outside {allowed_evidence}")
get_pattern = (evidence / "coma/get_pattern.coma").read_text()
false_pattern = (evidence / "coma/reject_false_contract.coma").read_text()
witness = (evidence / "coma/get_witness.coma").read_text()
false_witness = (evidence / "coma/reject_witness_contract.coma").read_text()
source = (fixture / "src/lib.rs").read_text()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"SHAPE CHECK FAILED: {message}")

for i, byte in enumerate((71, 69, 84, 32)):
    require(f"Slice64.get <UInt8.t> {{bytes}} {{({i}: UInt64.t)}}" in get_pattern,
            f"arbitrary-array positive is missing Slice64.get offset {i}")
    require(f"r = ({byte}: UInt8.t)" in get_pattern,
            f"arbitrary-array positive is missing pattern byte {byte}")

for i, byte in enumerate((71, 69, 84, 32)):
    require(f"Slice64.get <UInt8.t> {{bytes}} {{({i}: UInt64.t)}}" in false_pattern,
            f"frozen negative is missing Slice64.get offset {i}")
    require(f"r = ({byte}: UInt8.t)" in false_pattern,
            f"frozen negative is missing pattern byte {byte}")
match_branch = re.search(r"\| bb6 = s0(?P<branch>.*?)(?=\n  \[ & _ret:)", false_pattern, re.S)
require(match_branch is not None and "[ &_ret <- true ]" in match_branch.group("branch")
        and "return {_ret}" in match_branch.group("branch"),
        "matching bb6 branch no longer stores and returns true")
require("result = false" in false_pattern,
        "frozen negative postcondition is not false")
require("r = (32: UInt8.t)} (! bb6)" in false_pattern,
        "final 32-byte lane no longer guards the true-return branch")

for candidate, label in ((witness, "get_witness"),
                         (false_witness, "reject_witness_contract")):
    require("get_pattern " in candidate, f"{label} no longer calls the checked positive helper")
    for i, byte in enumerate((71, 69, 84, 32)):
        require(re.search(rf"Seq\.get __arr_temp\.Slice64\.elts {i} = .*\({byte}: UInt8\.t\)", candidate),
                f"{label} literal input lane {i} is not {byte}")
require("[@expl:get_witness ensures]" in witness and "result}" in witness,
        "ground witness does not promise a true result")
require("[@expl:reject_witness_contract ensures]" in false_witness and "result = false" in false_witness,
        "caller negative control does not promise a false result")
require("#[ensures(result)]\npub fn get_witness()" in source,
        "source witness contract changed")
require("#[ensures(result == false)]\npub fn reject_witness_contract()" in source,
        "source caller negative contract changed")

out = evidence / "ground-diagnostic-preparation"
out.mkdir(exist_ok=True)
formula = """(set-logic QF_UF)\n(declare-const result Bool)\n(assert (= result true))\n(assert (not (= result false)))\n(check-sat)\n"""
(out / "witness-vs-false-postcondition.smt2").write_text(formula)
report = [
    "# Ground negative diagnostic preparation",
    "",
    "Status: PREPARED_NOT_RUN. This is not a solver result and does not replace the frozen full negative VC timeout.",
    "",
    f"Translation evidence: `{evidence}`",
    "",
    "The arbitrary-input `get_pattern` body has constant-index reads 0..3 and pattern bytes 71, 69, 84, 32. Its nine goals were separately Valid in proof run `proof-20261005T171205Z-39275`.",
    "The frozen `reject_false_contract` COMA contains a matching branch assigning true while requiring result=false; its fifth VC timed out after 30 seconds with no counterexample. This status is retained unchanged.",
    "The new `get_witness` and `reject_witness_contract` translations both call only `get_pattern` with the fixed satisfying array, and the four literal lanes match the pattern. Translation and type-only checking passed for this bundle. The SMT file is a ground reduction of the helper-contract consequence; it has not been submitted to Z3 and must not be presented as the original VC's counterexample.",
    "",
    "| Artifact | SHA-256 |",
    "| --- | --- |",
]
for path in (fixture / "src/lib.rs", evidence / "coma/get_pattern.coma",
             evidence / "coma/reject_false_contract.coma", evidence / "coma/get_witness.coma",
             evidence / "coma/reject_witness_contract.coma", out / "witness-vs-false-postcondition.smt2"):
    report.append(f"| `{path.name}` | `{hashlib.sha256(path.read_bytes()).hexdigest()}` |")
(out / "REPORT.md").write_text("\n".join(report) + "\n")
print(f"shape checks passed; preparation only: {out}")
