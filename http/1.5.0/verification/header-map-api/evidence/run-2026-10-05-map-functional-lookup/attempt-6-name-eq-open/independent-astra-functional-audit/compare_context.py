#!/usr/bin/env python3
"""Compare complete independently printed task streams and frozen source bytes."""
import difflib
import gzip
import hashlib
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
ATTEMPT = HERE.parent
OLD = ATTEMPT.parent / "attempt-5"
OLD_AUDIT = OLD / "independent-astra-functional-audit"
sha = lambda data: hashlib.sha256(data).hexdigest()

def packed(path, data):
    output = gzip.compress(data, mtime=0)
    path.write_bytes(output)
    return {"path": path.name, "raw_sha256": sha(data), "raw_bytes": len(data), "gzip_sha256": sha(output)}

old_report = json.loads((OLD_AUDIT / "report.json").read_text())
report = json.loads((HERE / "report.json").read_text())
old_results = {r["target"]: r for r in old_report["results"]}
proof_summary = json.loads((ATTEMPT / "full15-proof-summary.json").read_text())
summary_proofs = {r["coma"].split("http_header_map_api_proof_rlib/", 1)[1]: r for r in proof_summary["proofs"]}
rows = []
for i, row in enumerate(report["results"]):
    old = old_results[row["target"]]
    old_bytes = gzip.decompress((OLD_AUDIT / old["stdout"]["path"]).read_bytes())
    new_bytes = gzip.decompress((HERE / row["stdout"]["path"]).read_bytes())
    assert sha(new_bytes) == row["stdout"]["raw_sha256"]
    summary = summary_proofs[row["target"]]
    assert summary["coma_sha256"] == row["coma_sha256"]
    assert summary["proof_json_sha256"] == row["proof_json_sha256"]
    diff = "".join(difflib.unified_diff(old_bytes.decode().splitlines(keepends=True), new_bytes.decode().splitlines(keepends=True), fromfile="attempt5 complete stdout", tofile="attempt6 complete stdout")).encode()
    rows.append({"target": row["target"], "old_stdout_sha256": sha(old_bytes), "new_stdout_sha256": sha(new_bytes), "byte_identical": old_bytes == new_bytes, "complete_diff": packed(HERE / f"context-{i:02}.diff.gz", diff)})

old_sources = {r["path"]: r for r in json.loads((OLD / "source-freeze.json").read_text())["sources"]}
sources = {r["path"]: r for r in json.loads((ATTEMPT / "source-freeze.json").read_text())["sources"]}
after = json.loads((ATTEMPT / "source-after.json").read_text())
assert set(old_sources) == set(sources)
assert after["changed_count"] == after["missing_count"] == 0
assert len(after["sources"]) == len(sources) == 177
for item in after["sources"]:
    assert item["sha256"] == item["current_sha256"] == sources[item["path"]]["sha256"]
    assert item["unchanged"]
source_diffs = []
for key, item in sources.items():
    if item["sha256"] != old_sources[key]["sha256"]:
        source_diffs.append({"path": key, "old_sha256": old_sources[key]["sha256"], "new_sha256": item["sha256"]})
        if key in {"http/1.5.0/src/header/map.rs", "http/1.5.0/src/header/name.rs"}:
            old_text = (OLD / "sources" / key).read_text()
            new_text = (ATTEMPT / "sources" / key).read_text()
            diff = "".join(difflib.unified_diff(old_text.splitlines(keepends=True), new_text.splitlines(keepends=True), fromfile="attempt5 " + key, tofile="attempt6 " + key))
            path = HERE / (Path(key).name + ".diff")
            path.write_text(diff)
            source_diffs[-1]["diff_file"] = path.name
            if key.endswith("name.rs"):
                needle = "impl PartialEqModel<ReprDeepModel<(Seq<u8>, bool)>> for ReprDeepModel<Seq<u8>> {\n    #[logic(open(self))]"
                assert old_text.replace(needle, needle.replace("open(self)", "open"), 1) == new_text

target = "header/map/as_header_name/impl_Sealed_for_ref_str/find.coma"
old_coma = (OLD / "comas" / target).read_text()
new_coma = (ATTEMPT / "comas" / target).read_text()
pattern = r"predicate eq_model_ReprDeepModel_Seq_u8[^\n]*"
old_decl = re.search(pattern, old_coma)[0]
new_decl = re.search(pattern, new_coma)[0]
assert not old_decl.endswith("=") and new_decl.endswith("=")
assert re.search(pattern + r"\n  \[%#[^]]+\] header_name_matches_hdr_name self rhs", new_coma)
assert report["root_count"] == report["complete_root_count"] == 37
assert report["terminal_leaf_count"] == report["proved_terminal_leaf_count"] == 40
result = {"solver_invoked": False, "task_stream_comparison": "Complete raw stdout bytes, without normalization or goal-only filtering", "task_streams": rows,
          "unchanged_stream_count": sum(r["byte_identical"] for r in rows), "changed_stream_count": sum(not r["byte_identical"] for r in rows),
          "old_to_new_source_deltas": source_diffs, "all_177_source_after_hashes_match_frozen_snapshot": True,
          "all_archived_proof_json_hashes_match_owner_summary": True,
          "name_source_change_exactly_open_self_to_open": True,
          "old_eq_model_declaration": old_decl, "new_eq_model_declaration": new_decl,
          "new_eq_model_body": "header_name_matches_hdr_name self rhs", "visible_bridge_confirmed": True}
(HERE / "context-report.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps({k: result[k] for k in ["unchanged_stream_count", "changed_stream_count", "visible_bridge_confirmed", "all_177_source_after_hashes_match_frozen_snapshot"]}))
