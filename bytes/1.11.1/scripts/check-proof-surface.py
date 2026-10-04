#!/usr/bin/env python3
"""Inventory the actual default-std compiler surface; no proof claims from counts."""
import argparse, csv, hashlib, json, pathlib, re

ROOT = pathlib.Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('rustdoc_json', type=pathlib.Path)
parser.add_argument('--check', action='store_true', help='check inventory identity without changing proof statuses')
args = parser.parse_args()
doc = json.loads(args.rustdoc_json.read_text())
parents = {}
for item in doc['index'].values():
    inner = item['inner']
    if 'impl' in inner:
        info = inner['impl']
        label = json.dumps({'for': info['for'], 'trait': info['trait']}, sort_keys=True)
        for ident in info['items']:
            parents[ident] = label
    if 'trait' in inner:
        for ident in inner['trait']['items']:
            parents[ident] = item['name']
rows = []
for item in doc['index'].values():
    span = item.get('span')
    kind = next(iter(item['inner']))
    if not span or item['crate_id'] != 0 or kind not in ('function', 'trait', 'impl', 'static', 'constant', 'macro'):
        continue
    source = ROOT / span['filename']
    if not source.resolve().is_relative_to(ROOT / 'src') or not source.is_file():
        continue
    owner = parents.get(item['id'], '')
    name = item['name'] or kind
    identity = json.dumps([span['filename'], span['begin'], name, owner], sort_keys=True)
    row = dict(item_id=hashlib.sha256(identity.encode()).hexdigest()[:16], kind=kind,
               item=name, owner=owner, visibility=json.dumps(item['visibility']),
               source_path=span['filename'], source_span=f"{span['begin'][0]}:{span['begin'][1]}-{span['end'][0]}:{span['end'][1]}",
               source_hash=hashlib.sha256(source.read_bytes()).hexdigest(), configuration='std;x86_64-unknown-linux-gnu',
               contract_id='', functional_status='not_started', memory_status='not_started', concurrency_status='not_started',
               panic_status='not_started', implementation_status='not_started', assumption_ids='', proof_artifact='',
               next_action='review contract and runtime dependencies')
    rows.append(row)
rows.sort(key=lambda r: (r['source_path'], r['source_span'], r['item_id']))
path = ROOT / 'PUBLIC_API_COVERAGE.csv'
if args.check:
    old = list(csv.DictReader(path.open()))
    key = lambda r: (r['item_id'], r['source_hash'])
    if {key(r) for r in old} != {key(r) for r in rows}:
        raise SystemExit('source/item surface changed; update coverage and review new obligations')
else:
    old = {r['item_id']: r for r in csv.DictReader(path.open())} if path.exists() else {}
    for row in rows:
        previous = old.get(row['item_id'])
        if previous and previous['source_hash'] == row['source_hash']:
            for field in ('contract_id', 'functional_status', 'memory_status', 'concurrency_status', 'panic_status',
                          'implementation_status', 'assumption_ids', 'proof_artifact', 'next_action'):
                row[field] = previous[field]
    with path.open('w') as f:
        writer = csv.DictWriter(f, fieldnames=list(rows[0]), lineterminator="\n"); writer.writeheader(); writer.writerows(rows)
    # Bootstrap candidate list. Every unsafe token must be reviewed into semantic obligations.
    # It intentionally includes comment candidates rather than silently dropping obligations.
    unsafe = []
    for source in sorted((ROOT/'src').rglob('*.rs')):
        if source.name == 'verification.rs':
            continue  # legacy model is not part of the runtime configuration
        for line, text in enumerate(source.read_text().splitlines(), 1):
            if re.search(r'\bunsafe\b', text):
                unsafe.append(dict(obligation_id=f'{source.relative_to(ROOT)}:{line}', source_location=f'{source.relative_to(ROOT)}:{line}',
                    operation=text.strip(), required_permission='review', initialization_condition='review',
                    layout_or_provenance_condition='review', ordering='review', discharging_lemma='', proof_artifact='', status='not_started'))
    with (ROOT/'UNSAFE_LEDGER.csv').open('w') as f:
        writer=csv.DictWriter(f,fieldnames=list(unsafe[0]),lineterminator="\n");writer.writeheader();writer.writerows(unsafe)
    manifest=dict(rustdoc_format=doc['format_version'],rustdoc_sha256=hashlib.sha256(args.rustdoc_json.read_bytes()).hexdigest(),
                  configuration='std;x86_64-unknown-linux-gnu',compiler_item_count=len(rows),
                  coverage_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                  unsafe_ledger_note='syntactic candidates, including comments; semantic review remains required')
    (ROOT/'verification/artifacts/surface-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(f'{len(rows)} compiler items inventoried; statuses are independent of inventory membership')
