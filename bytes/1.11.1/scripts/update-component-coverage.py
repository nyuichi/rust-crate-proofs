#!/usr/bin/env python3
"""Attach isolated exact-source body evidence to a fresh compiler inventory.

This never marks a Bytes handle/trait caller proved merely because it calls a
proved helper. Imported dependencies can appear in several component proofs.
"""
import csv
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
summary = json.loads((root / 'verification/artifacts/component-evidence.json').read_text())
proved = {}
for component in summary['components']:
    sources = component['inputs']['direct_path_sources']
    for source, digest in sources.items():
        if hashlib.sha256((root / source).read_bytes()).hexdigest() != digest:
            raise SystemExit(f'source changed after proof: {source}')
        module = Path(source).stem
        for artifact in component['proof_artifacts']:
            path = Path(artifact['path'])
            if path.suffix == '.coma' and path.parent.name == module:
                proved.setdefault((source, path.stem), component['target'])
coverage = root / 'PUBLIC_API_COVERAGE.csv'
with coverage.open() as stream:
    reader = csv.DictReader(stream)
    fields = reader.fieldnames
    rows = list(reader)
updated = 0
for row in rows:
    # Normal pointer_addr uses a cast; the probe proves its cfg(creusot) addr() variant.
    if row['source_path'] == 'src/provenance_specs.rs' and row['item'] == 'pointer_addr':
        continue
    target = proved.get((row['source_path'], row['item']))
    if target and row['kind'] == 'function':
        row.update(
            contract_id=f'{target}::{row["item"]}',
            functional_status='body_proved_under_contract',
            memory_status='body_proved_under_contract',
            concurrency_status='not_applicable_to_this_helper',
            panic_status='body_proved_under_contract',
            implementation_status='isolated_exact_source_body_proved',
            assumption_ids='Creusot-0.13-standard-contracts;reviewed-standard-extensions-in-TRUSTED_BASE.md',
            proof_artifact=f'verification/artifacts/component-evidence/{target}/component.json',
            next_action='prove actual handle/trait callers and their resource premises; full runtime remains unproved',
        )
        updated += 1
with coverage.open('w') as stream:
    writer = csv.DictWriter(stream, fieldnames=fields, lineterminator='\n')
    writer.writeheader()
    writer.writerows(rows)
manifest_path = root / 'verification/artifacts/surface-manifest.json'
manifest = json.loads(manifest_path.read_text())
manifest['coverage_sha256'] = hashlib.sha256(coverage.read_bytes()).hexdigest()
manifest['isolated_exact_source_function_rows'] = updated
manifest['coverage_note'] = 'body proofs are conditional on their contracts; caller integration is not implied'
manifest_path.write_text(json.dumps(manifest, indent=2) + '\n')
print(f'{updated} helper function rows linked to isolated exact-source body proofs')
