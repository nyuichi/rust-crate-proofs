#!/usr/bin/env python3
"""Compare COMA bodies while removing only compiler-generated source coordinates."""
import hashlib
import json
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[7]
EVIDENCE = ROOT / "http/1.5.0/verification/uri/evidence/uri-eq-transport-proof-2026-10-05"
COORDINATE = re.compile(r'("[^"\\\n]+") \d+ \d+ \d+ \d+')
PAIRS = [
    ("conversion", "from_authority_for_uri.coma", "uri/impl_From_for_Uri/from.coma"),
    ("conversion", "from_authority_for_uri__refines.coma", "uri/impl_From_for_Uri/from__refines.coma"),
    ("conversion", "from_path_and_query_for_uri.coma", "uri/impl_From_for_Uri_0/from.coma"),
    ("conversion", "from_path_and_query_for_uri__refines.coma", "uri/impl_From_for_Uri_0/from__refines.coma"),
    ("debug", "uri_debug_fmt.coma", "uri/impl_Debug_for_Uri/fmt.coma"),
    ("debug", "uri_debug_fmt__refines.coma", "uri/impl_Debug_for_Uri/fmt__refines.coma"),
]

def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def normalized(data: bytes) -> bytes:
    text = data.decode()
    return COORDINATE.sub(r'\1 <source-span>', text).encode()

artifacts = []
for kind, name, fresh_suffix in PAIRS:
    old_root = EVIDENCE.parent / f"uri-{kind}-proof-2026-10-05"
    old_path = old_root / "comas" / name
    fresh_path = ROOT / "http/1.5.0/verification/uri/verif/http_uri_proof_rlib" / fresh_suffix
    old, fresh = old_path.read_bytes(), fresh_path.read_bytes()
    old_norm, fresh_norm = normalized(old), normalized(fresh)
    artifacts.append({
        "prior_evidence": str(old_path.relative_to(ROOT)),
        "fresh_emission": str(fresh_path.relative_to(ROOT)),
        "archived_sha256": sha(old),
        "fresh_sha256": sha(fresh),
        "exact_bytes_match": old == fresh,
        "archived_normalized_sha256": sha(old_norm),
        "fresh_normalized_sha256": sha(fresh_norm),
        "normalized_match": old_norm == fresh_norm,
        "normalization": "Replaces only source-span coordinates in generated span annotations; all source paths and COMA logic remain byte-compared.",
    })

body = EVIDENCE / "comas/uri_eq_body.coma"
refines = EVIDENCE / "comas/uri_eq_refines.coma"
obj = {
    "scope": "Fresh clean emission and source-span-normalized comparison of six previously accepted conversion/Debug inputs.",
    "emission_exit_code": int((EVIDENCE / "emission/exit-status.txt").read_text().strip()),
    "six_prior_conversion_and_debug_comas_exact_bytes_match": all(x["exact_bytes_match"] for x in artifacts),
    "six_prior_conversion_and_debug_comas_match_after_source_span_normalization": all(x["normalized_match"] for x in artifacts),
    "normalizer": "emission/normalize_compare.py",
    "prior_comas": artifacts,
    "current_same_type_eq_body": {
        "fresh_emission": "verif/http_uri_proof_rlib/uri/impl_PartialEq_for_Uri/eq.coma",
        "archive": "comas/uri_eq_body.coma",
        "sha256": sha(body.read_bytes()),
    },
    "current_same_type_eq_refines": {
        "fresh_emission": "verif/http_uri_proof_rlib/uri/impl_PartialEq_for_Uri/eq__refines.coma",
        "archive": "comas/uri_eq_refines.coma",
        "sha256": sha(refines.read_bytes()),
    },
}
(EVIDENCE / "emission/coma-hash-comparison.json").write_text(json.dumps(obj, indent=2) + "\n")
print(json.dumps({"exact_matches": obj["six_prior_conversion_and_debug_comas_exact_bytes_match"], "normalized_matches": obj["six_prior_conversion_and_debug_comas_match_after_source_span_normalization"]}, indent=2))
