#!/usr/bin/env python3
"""Summarize only current kernel receipts and the separate strict API audit."""
import collections, hashlib, json
from pathlib import Path
R = Path(__file__).resolve().parent
read = lambda p: json.loads(p.read_text())
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
result = read(R/'results.json')
result['status'] = 'partial functional verification; whole-crate completion blocked by recorded translation/typechecking failures and remaining specifications'
result['native']['receipt'] = 'evidence/native-result.json'
result['native']['theorem_count_including_helpers'] = sum(len(v['theorems']) for v in read(R/'evidence/native-result.json')['proof_files'].values())
extensions = []
for path in sorted((R/'extensions').glob('*/manifest.json')):
    folder = path.parent
    receipt = read(folder/'receipt.json')
    assert receipt['status'] == 'kernel_checked'
    for name, expected in receipt['files'].items():
        assert sha(folder/name) == expected, f'Stale {folder.name}/{name}'
    extensions.append({'group':folder.name,'mapped_theorem_claim_count':len(receipt['theorems']),
      'receipt':str((folder/'receipt.json').relative_to(R)),
      'specifications':str(path.relative_to(R)),
      'axiom_log':str((folder/'audit.log').relative_to(R))})
result['native']['extensions'] = extensions
result['native']['extension_mapped_theorem_claim_count'] = sum(x['mapped_theorem_claim_count'] for x in extensions)
result['native']['total_theorem_claims_including_helpers'] = result['native']['theorem_count_including_helpers'] + result['native']['extension_mapped_theorem_claim_count']
result['native']['count_note'] = 'These are theorem claims, not distinct APIs or fully specified functions; use the coverage ledger for source-body/property mapping.'
result['native']['reproduce'] = 'bash /workspace/bytes-aeneas-fresh/verification/verify-all.sh --reextract'
audit = read(R/'elicit-integration/audit-selected-artifacts/receipt.json')
assert audit['report']['stable']
counts = collections.Counter(x['outcome'] for x in audit['report']['results'])
result['cargo_elicit']['accepted_api_count'] = counts['success']
result['cargo_elicit']['unresolved_api_count'] = counts['unresolved']
result['cargo_elicit']['goals'] = audit['report']['results']
result['cargo_elicit']['fresh_audit_receipt'] = 'elicit-integration/audit-expanded-final-receipt.json'
result['cargo_elicit']['strict_axiom_policy_changed'] = False
result['coverage'] = read(R/'coverage/summary.json')
result['coverage']['ledger'] = 'coverage/ledger.json'
result['toolchain'] = 'toolchain.json'
result['frozen_boundary'] = 'opaque-baseline.json'
result['whole_crate_typecheck'] = {'status':'failed','log':'backend/whole-generated-lean.log',
 'blockers':['Mutable-reference backward-state continuations do not match trait dictionary signatures.',
 'Generic chain/chain_mut defaults do not match expected trait dictionary parameters.',
 'AtomicMut.with_mut destructures a generic callback result as a product.'],
 'name_collision':'A binder-hygiene translator patch fixes from_static namespace shadowing; actual source constructors now separately kernel checked.',
 'monomorphize_mut':'Two 180-second extraction runs timed out before LLBC; effect on this failure is inconclusive.'}
result['functional_results'] = [
 'Bytes/BytesMut getters return translated fields; constructors store explicitly supplied primitive pointer/atomic results and establish selected length properties.',
 'Take/Limit/Chain/IntoIter constructor, accessor, state, bound and branch properties are mapped in their manifests; callback success/coherence assumptions are explicit.',
 'Fixed-width integer writes append the specified endian byte sequence if put_slice satisfies the explicit append law and succeeds; float wrappers are conditional on bit conversion and integer writer contracts.',
 'With enough input and explicit coherent prefix-chunk/advance laws, try_copy_to_slice copies exactly the initial view prefix and consumes that many bytes across multiple chunks.',
 'Sign extension succeeds for all byte widths1..8 and equals low-byte truncation followed by sign extension to64bits. Capacity decode0..7 and64-bit encoding bounds/formula are proved.',
 'Safe UninitSlice range selection and mutable backward updates are conditional on conversion callbacks; inclusive ranges additionally depend on opaque core SliceIndex operations.',
 'Self-comparisons, constructor/metadata operations, reserve/reclaim/unsplit branches and other core properties are documented per theorem; delegated behavior remains explicit.',
 'Vec::put appends the full modeled source under coherent chunk/advance laws, a total isizeMAX size bound, and successful content-preserving reserve; actual Vec model append steps are proved.',
 'TryGetError equality compares both counts. Panic helpers produce the panic result under the Aeneas formatting model, which erases message rendering.',
 'Bytes FromIterator and the private pointer helper compose explicitly assumed primitive results; these claims do not establish iteration or pointer-provenance semantics.',
 'Borrowed IntoIterator for Bytes and BytesMut returns the exact modeled slice iterator at index zero, conditional on the successful frozen as_slice result; subsequent iteration semantics are outside these claims.',
 'Instantiating translated copy and Vec put with explicit finite-slice Lean dictionaries proves concrete prefix/suffix copy and ordered multi-chunk append without generic source-callback hypotheses. These dictionaries model only the exercised operations, not particular Rust Buf implementations; Vec reserve success/content preservation and size bounds remain explicit.',
 'The translated put_u32_le appends exactly four specified little-endian bytes in a bounded-list Lean writer model when capacity is sufficient; this does not identify that model with a concrete Rust BufMut implementation.',
 'BytesMut reserve preserves its exact slice observation when the request fits existing capacity. Cross Bytes/BytesMut equality is symmetric under explicit successful view/index calls. Byte-list lexicographic comparison is defined and proved equal to List.compareLex, while its connection to opaque standard slice partial_cmp remains an explicit hypothesis.'
]
result['assumptions_and_limits'] = [
 'The verified configuration is default/std on x86_64-unknown-linux-gnu; serde, no_std and other targets are outside this run.',
 'Results apply to the recorded adapted1.11.1 source. Whole-release semantic equivalence to crates.io bytes1.11.1 is not proved.',
 'The original209 metadata opacity patterns and current Rust source are frozen and hash checked. Unsafe and other opaque implementations remain trusted.',
 'Each theorem states a particular property; a branch or forwarding theorem is not a complete API specification.',
 'Ordering comparison claims delegate to the opaque standard comparison model; they do not prove its lexicographic semantics.',
 'Callback equations, content laws and representation operations listed in manifests are premises, not proofs of the callees.',
 'Opaque type and function dependencies are retained in exact axiom logs. FloatTypes introduces only uninterpreted F32/F64 types.',
 'Generated function bodies are unchanged. Recorded extractions add only the FloatTypes import; scoped exclusions, when needed, are explicit in extraction recipes.',
 'Rust compilation, Charon/Aeneas translation, standard library models and the reviewed translator patch are trusted.',
 'Allocation, raw memory validity, refcounts/concurrency and global panic freedom are not established.',
 'The full generated crate module still fails Lean typechecking; the verified extractions and all uncovered source bodies are recorded separately.',
 'The original upstream source and old Creusot work were not edited.'
]
(R/'results.json').write_text(json.dumps(result,indent=2)+'\n')
print('results.json updated from current receipts')
