# base16ct1.0.0 strict source inventory reconciliation

Original default/all/minimal inventory and rustdoc files remain unchanged. This
source-only view uses their exact raw rows, implementations, source spans, cfg,
generic bounds and provided methods. No new proof, compiler build or server
operation is performed. Bounds not explicitly resolved remain conditional.

| Configuration | Total ledger records | Callable+default raw rows | Shadowed defaults (not extra bodies) | Derived std impl rows excluded | Ordinary conditional candidates | Consumer nightly-gated candidates |
|---|---:|---:|---:|---:|---:|---:|
| default |119|56|1|14|31|10|
| minimal |119|56|1|14|31|10|
| all (alloc) |138|69|3|14|42|10|

Thus distinct call signatures before derive exclusion are55/55/66, and the
non-derived conditional candidate totals are41/41/52. Total119/138 includes
structural and implementation relationships and is not a callable API count.
Source and type bounds can still limit which concrete instantiations are callable;
these numbers do not assert arbitrary-type/argument availability.

## Shadow and derive origin

Default/minimal incorrectly emit inherited Eq::assert_fields_are_eq for HexDisplay
in addition to its actual generated override. All additionally emit inherited
ToOwned::clone_into despite the actual std blanket overrides for Error and
HexDisplay. shadow-exact-rows.json preserves raw rows/impls/method lists and the
actual override names; do not count those1/3 defaults as independent bodies.

Under the user exemption for std traits declared by derive, the whole specific
automatically_derived implementation is excluded, including its non-overridden
defaults. Seven explicit generated function rows plus seven distinct inherited
defaults produce14 exclusions. The shadow is removed before that count. A type
merely having some derives does not exempt its separate hand-written impls:
Error's manual core::error::Error defaults remain in scope, as do its Display,
From and HexDisplay's manual formatting impls. No nonstandard-derived callable
was found; such a derive would not be excluded by this policy.

## Trait gates and blanket binders

Effective gates combine method and defining-trait metadata. In particular,
core::io::size_hint::SizeHint is hidden/unstable(core_io_internals): its inherited
size_hint method cannot be advertised as ordinarily callable just because the
method row lacks its own unstable attribute. CloneToUninit and Error's internal
type_id/provide gates remain separate too. Error::type_id also has private::Internal
an argument whose public construction/privacy is unresolved; its gated/internal
entry is not promoted into unconditional callable coverage. Cargo all-features
only enables alloc, not arbitrary consumer #![feature] gates.

Reflexive blanket From/Borrow/BorrowMut binds the std template T to concrete Self
(Error or HexDisplay<'a>), not a new payload type. strict-rows.json preserves raw
impl patterns, corrected identity displays and source/target contexts. Generic
where clauses, AsRef conversions and other callbacks remain conditional rather
than mechanically solved for every input type.

## Three independent evidence columns

Existing machine3 are uniquely source-bound to encoded_len, decoded_len and
From<Error> for core::fmt::Error. Their keys/theorems remain unchanged; default
feature proof scope is retained, and encoded_len above usize::MAX/2 remains
outside the proved precondition. Other configurations are recorded as source
correspondence/reference, not extra independent runs.

The Display1910 supplement is separately limited to both Error variants, default
{} and the specified initialized64 writer accepting all writes or failing before
output. Its corrected literal lengths are23/21; the historical22/20 metadata
erratum is retained in that packet. It is not general Formatter correctness.

Codec1484 is separately limited to lower/upper/mixed decode on ordinary &[u8]
AsRef input length2 with full dst length3, exact Ok/InvalidEncoding classification,
Ok returned length1/value, suffix indices1/2 unchanged on Ok/Err. Err dst0 is
unspecified. Lower/upper encode uses input1/dst4, returns exact ASCII length2,
preserves suffix2/3, and roundtrips through actual decode into dst1. Arbitrary
AsRef implementations, lengths/capacities, allocation and encode_str/unchecked
UTF8 are not thereby proved. Neither bounded supplement adds Aeneas claims.

The default14 manual functions consist of machine3, Error Display1 bounded,
codec5 bounded and5 other unproved manual formatting/encode_str functions.
All-features adds5 allocation helpers, still unproved. None of these counts
closes external defaults/blankets or the generic API domains. Trait/core models,
source compiler/target and each packet's exact Rust version remain authoritative.

## Collection flags differ from the older three-crate ledgers

These base16 inventories explicitly document hidden items. The supplied older
endian/nohash/normalize provenance rustdoc commands omit that flag. Therefore
strict3crate189/50/80/128 are strict applicable counts **within the preserved
supplied ledgers**, not demonstrated exhaustive counts over the same full
hidden/nightly catalogue as base16. No new collection was run to resolve that
scope difference. This does not affect the unique existing14/66/1 source bindings.
See THREE-CRATE-COLLECTION-SCOPE-ADDENDUM.md for a superseding documentation note.

reconcile.py regenerates this view without modifying original inputs. Full raw
rows, classifications, evidence columns, trait gate sources and input hashes
are retained. This is not live publication state or full crate closure.
