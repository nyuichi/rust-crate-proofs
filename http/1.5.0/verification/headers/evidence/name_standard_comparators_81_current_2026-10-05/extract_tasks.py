from pathlib import Path
import hashlib
import json
import shutil
import subprocess

run = Path(__file__).resolve().parent
name_dir = run / "emission/verif/http_headers_proof_rlib/header/name"
targets = sorted(name_dir.glob("bytes_equal_*.coma"))
targets += [
    name_dir / "bytes_equal.coma",
    name_dir / "standard_header_absence_case.coma",
    name_dir / "impl_StandardHeader/from_bytes.coma",
    name_dir / "selected_standard_header_comparator_consumer.coma",
    name_dir / "impl_HdrName/from_bytes.coma",
    name_dir / "impl_HeaderName/from_bytes.coma",
]
if len(targets) != 87 or any(not path.is_file() for path in targets):
    raise SystemExit(f"expected 87 relevant COMAs; found {len(targets)}")

extract_root = run / "why3_tasks"
extract_root.mkdir(exist_ok=True)
why3_log = []
rows = []
for coma in targets:
    key = coma.relative_to(run / "emission").as_posix().replace("/", "__").removesuffix(".coma")
    dest = extract_root / key
    if dest.exists():
        shutil.rmtree(dest)
    dest.mkdir(parents=True)
    command = [
        "why3", "prove",
        "-C", "/workspace/proof-tools/config/creusot/why3.conf",
        "-L", "/workspace/proof-tools/creusot-data/share/why3find/packages/creusot",
        "-a", "split_vc", "-D", "why3", "-o", str(dest), str(coma),
    ]
    result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    (dest / "extract.log").write_text(result.stdout)
    task_files = sorted(dest.glob("*.why"))
    if result.returncode != 0:
        raise SystemExit(f"Why3 extraction failed for {coma}: exit {result.returncode}; see {dest}/extract.log")
    if not task_files:
        raise SystemExit(f"Why3 extraction produced no tasks for {coma}")
    why3_log.append(f"### {key} exit={result.returncode} tasks={len(task_files)}\n{result.stdout}")
    rows.append({
        "target": key,
        "coma": coma.relative_to(run).as_posix(),
        "coma_sha256": hashlib.sha256(coma.read_bytes()).hexdigest(),
        "direct_task_count": len(task_files),
        "tasks": [
            {"path": path.relative_to(run).as_posix(), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
            for path in task_files
        ],
    })
(run / "why3_task_extraction.log").write_text("\n".join(why3_log))
ledger = [
    f"{task['sha256']}  {task['path']}"
    for row in rows
    for task in row["tasks"]
]
(run / "why3_tasks.sha256").write_text("\n".join(sorted(ledger)) + "\n")
report = {
    "method": "Why3 prove -a split_vc -D why3 -o; solver-free task extraction from the frozen COMAs",
    "solver_invoked": False,
    "exit_code": 0,
    "targets": len(rows),
    "total_direct_tasks": sum(row["direct_task_count"] for row in rows),
    "targets_summary": [
        {"target": row["target"], "direct_task_count": row["direct_task_count"], "coma_sha256": row["coma_sha256"]}
        for row in rows
    ],
    "task_ledger": "why3_tasks.sha256",
    "task_ledger_sha256": hashlib.sha256((run / "why3_tasks.sha256").read_bytes()).hexdigest(),
    "full_log": "why3_task_extraction.log",
    "full_log_sha256": hashlib.sha256((run / "why3_task_extraction.log").read_bytes()).hexdigest(),
}
(run / "why3_task_arities.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({"targets": len(rows), "tasks": report["total_direct_tasks"], "counts": {row['target']:row['direct_task_count'] for row in rows if row['direct_task_count'] != 1}}, indent=2))
