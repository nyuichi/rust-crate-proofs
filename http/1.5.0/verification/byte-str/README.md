# ByteStr proof checkpoint

The target includes the actual `src/byte_str.rs` and the HTTP re-export of the
single `creusot-std` Bytes observer. Its `View` is the exact byte sequence and
its invariant is `valid_utf8(self@)`. Constructor, conversion, borrow, clone,
and equality contracts use that same sequence. The proof depends on the
completed-Bytes premise recorded in `verification/bytes/README.md` and on the
existing trusted `utf8_error_matches` contract in `creusot-std::std::string`
for the error branch of `str::from_utf8`.

The bundled Creusot prelude has a Unicode boundary limitation: its `char`
integer model uses `< 0x10FFFF`, excluding valid Rust `char` U+10FFFF. Thus
the UTF-8 model/proofs do not faithfully cover strings containing that scalar
value; this is an upstream prelude limitation, not an HTTP body result.

Run the focused target from this directory:

```text
../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove -- --locked --offline
```

The latest normal-mode run proved `new` (4 VCs), `from_static` (3), checked
`from_utf8` (4), `from_utf8_unchecked` including its actual debug check (5),
`Deref` (3), `Clone` (2 plus 1 refinement VC), `PartialEq` (2 plus 1
refinement VC), `PartialOrd` (2 plus 1 refinement VC), and `Ord` (2 plus 1
refinement VC). The `From<String>`, `From<&str>`, and `From<ByteStr> for Bytes`
bodies and refinements also pass. `new` uses two ghost assertions to provide
the empty character sequence as an explicit witness that empty bytes are valid
UTF-8. The manual `Debug` implementation uses the standard `DebugStruct`
builder and proves its body (13 VCs) and formatter-extension refinement (3
VCs), preserving the derive-style formatting path. The shared formatter model
establishes append-only output across the builder borrow, including each
field's `Debug::fmt` callback; it does not prove the exact formatted bytes.

`Hash::hash` also proves (4 VCs) that the generic hasher invariant is restored
after hashing. This is conditional on the completed `Bytes` dependency model
and the shared `Hasher` protocol: `write`, `write_usize`, and
`write_length_prefix` preserve any declared hasher invariant. The default
forwarding wrappers' specification bodies have separate proofs, and a positive
`EvenHasher` probe passes its body, refinement, and ByteStr caller. A negative
`OddMakingHasher` probe that changes an even state to an odd state is rejected
by its `write` body VC. Its `write` refinement VC alone passes because mutable
reference refinement uses a final-state prophecy; acceptance therefore checks
both body and refinement evidence. These results establish invariant
preservation only: neither digest contents nor Hash/Eq consistency are modeled.

The std Hash wrapper proofs are captured under `evidence/creusot-std/hash`.
They are specification-wrapper proofs against the pinned core method shapes,
not source-included proofs of the Rust core implementation. Matching the
pinned callback call graph and the `Hasher` override protocol remain explicit
standard-library premises. The concrete `Bytes::hash` state-preservation
contract is part of the completed-Bytes dependency premise. The callback and
formatter audits used Rust core commit
`6a979b3e32522049d0acb4a47f7ae44b7c8abfd5`.

The supporting `Seq<char>::to_bytes` definition in `creusot-std` is now a
direct head/tail recursion with a variant and an exact postcondition equating
it to the former `flat_map` model. Its actual owner-crate VC proves the body,
termination, and equivalence in one goal. Reproduce it from `creusot-libs` with:

```text
CARGO_NET_OFFLINE=false ../http/1.5.0/scripts/run-proof.sh cargo creusot -p creusot-std prove 'std::string::*' -- --features bytes-model --locked
```

The captured `.coma` and `proof.json` are under
`evidence/creusot-std/string-to-bytes/`. The exact `impl Seq<char>` source
block used for this proof has SHA-256
`1c2a444b6c594a9a96788ed371949be772cbfa884c404b7357e95c109a63e788`.

`evidence/manifest.json` records source fingerprints, exact Why3 target and
goal names, leaf VC counts, and workspace-root-relative proof artifact paths.
The HeaderName `Borrow` consumer outputs are retained there but quarantined: a
Astra found that relevant `vc_refines`/DeepModel goals reduce to literal
`true` because of a translator issue. The outputs are not counted as accepted
proof evidence until that issue is resolved.

The shared `Display for str` specification wrapper also has a focused proof
under `evidence/creusot-std/fmt`. It delegates to `Formatter::pad`; append-only
behavior of `pad` and the Debug builder methods is an audited std contract
premise, not a proof of exact output or a source-included proof of core fmt.

The unsafe constructor retains the real debug validation. Its panic message
is now a static string instead of including the `Utf8Error` and byte dump, so
invalid unsafe calls still panic in debug builds but expose different panic
text; the documented UTF-8 precondition remains required.
