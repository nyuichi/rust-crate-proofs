# Monomorphic seed LLBC dangling array-as-slice declaration

**Finding:** this is an unsupported Charon option combination admitted by a
preset-validation-order bug in the pinned version, not a missing decoder proof.
No original Rust/LLBC was edited, no new extraction/heavy build or model was run,
and no server operation was performed. Existing probe artifacts were read only.

## Observed minimal existing case

Both existing original SplitMix64::from_seed root probes contain only Fn11
(original constructor) and Fn12 (opaque read_words instantiated u64,N1), yet
Fn11 calls Fn0 and item_names identifies Fn0 as core::array::_::as_slice.
Both files have monomorphize=true **and** ops_to_function_calls=true,
index_to_function_calls=true and no_gen_tuple_structs=true. The include probe
explicitly includes core::array::_::as_slice but still emits no Fn0 declaration.
`findings.json` independently checks both declaration/reference maps and hashes.

The original coercion at splitmix64.rs79 is sufficient to explain the missing
edge: `let state: [_;1] = utils::read_words(&seed);`, where seed is [u8;8] and
read_words expects &[u8]. No decoder implementation/model is needed for the
failure: Aeneas stops in PrePasses.remove_unreachable at line106 when looking
up the absent function before external model generation.

A further standalone reduction to test upstream would be:

```rust
pub fn coercion(seed: [u8;8]) -> usize {
    let slice: &[u8] = &seed;
    slice.len()
}
```

That proposed reduction was **not compiled or claimed as reproduced** here.
The two supplied original-root probe logs/LLBC are the executed reproductions.
A configuration-only check of the installed binary was run, without source or
Cargo build: explicit `charon cargo --monomorphize --ops-to-function-calls`
rejects immediately with the incompatibility error in
`explicit-incompatible-options.log`.

## Root cause in pinned Charon 08d7c9d50144bd84848212c24701253585e90d1f

1. options.rs563-576: Aeneas preset sets ops/index lowering and disables generated
   tuple structs. options.rs678-688 explicitly rejects these choices with full
   monomorphization. Thus Aeneas preset + full mono is not a supported config.
2. Single-target main.rs211-218 validates raw options **before** the driver
   applies the preset at driver.rs233. The generated flags therefore evade
   validation. Explicit flags are rejected, explaining the different behavior.
3. translate_crate.rs271-283 registers the needed array helpers **polymorphically**,
   allocating IDs/names. translate_meta.rs992-996 makes non-trait polymorphic
   items Invisible in mono mode; translate_items.rs91-94 skips Invisible items.
   Fn0 is not removed from a formerly valid function map: its name/ID survives,
   while its declaration is intentionally not emitted in that mode.
4. builtins_to_function_calls.rs631-637 resolves candidates using item_names,
   not declaration existence. Lines668-674 retain the unique Fn0 name match;
   lines93-133 convert the reference-array unsizing into a call to that ID.
   This unsizing arm lacks a declaration-existence check (some indexing arms
   in the same pass do check declarations). The result is the dangling edge.
5. Aeneas PrePasses.ml106 performs a required Map lookup and rejects the malformed
   call graph. Restoring an external model alone cannot repair this early stage.

The exact inspected source ranges and hashes are retained in findings.json.
Source links for review:

- [preset and validation](https://github.com/AeneasVerif/charon/blob/08d7c9d50144bd84848212c24701253585e90d1f/charon/src/options.rs#L563)
- [single-target validation](https://github.com/AeneasVerif/charon/blob/08d7c9d50144bd84848212c24701253585e90d1f/charon/src/bin/charon/main.rs#L211)
- [builtin registration](https://github.com/AeneasVerif/charon/blob/08d7c9d50144bd84848212c24701253585e90d1f/charon/src/bin/charon-driver/translate/translate_crate.rs#L278)
- [mono opacity](https://github.com/AeneasVerif/charon/blob/08d7c9d50144bd84848212c24701253585e90d1f/charon/src/bin/charon-driver/translate/translate_meta.rs#L992)
- [unsize lowering](https://github.com/AeneasVerif/charon/blob/08d7c9d50144bd84848212c24701253585e90d1f/charon/src/transform/simplify_output/builtins_to_function_calls.rs#L93)

## Known upstream fix and its limit

Upstream commit
[f679421a5946296f1f6628dcb67907bafece2639](https://github.com/AeneasVerif/charon/commit/f679421a5946296f1f6628dcb67907bafece2639),
"Apply presets before validation options" (committed2026-10-17 UTC), moves
preset application ahead of validation for single-target cargo/rustc paths.
`upstream-validation-fix.patch` preserves the exact existing Git object.
This fixes admission/diagnostics, **not** support for mono+Aeneas lowering.
With the full Aeneas preset it may first reject no_gen_tuple_structs+mono;
after that incompatibility, ops/index+mono is independently rejected too.

The cached upstream origin/main is a5443ed4e648b153ea0f052e9c22a3667ef1c0b9
(2026-11-03); it includes the fix and retains the full-mono incompatibility gate.
Its lowering still selects named builtin IDs in the inspected code. No newer
network fetch or executable upgrade/build was performed. We therefore claim a
known upstream validation fix exists, not exhaustive knowledge of Nov5 branches
or a tested drop-in upgrade compatible with the pinned Aeneas LLBC schema.

## Supported-option alternatives examined

| Choice | What it changes | Why it is not a demonstrated seed verification workaround |
|---|---|---|
| Include exact generic as_slice root | Name/opacity selection | Mono metadata forces non-trait poly Invisible, and normal root enqueue skips generic items; existing include probe already fails |
| Use ordinary non-mono Aeneas preset | Retains generic builtin declaration, avoids this unsupported combination | Does not provide the desired concrete Word-dictionary-free decoder; generic helper obligation remains separate |
| Full mono without ops/index lowering (manual supported flags) | Avoids introducing this invalid Fn0 call | Leaves `&[u8;8] -> &[u8]` unsize. Pinned Aeneas InterpExpressions.ml697-707 accepts only Box-based structural unsizing in that path; this reference coercion is not a supported substitute. Source-inspected, not newly executed |
| Full mono with another preset/tests mode | Can avoid the incompatible lowering | Does not establish an Aeneas-compatible reference-unsizing/indexing pipeline; tests preset deliberately avoids ops/index in mono mode |
| Partial --monomorphize-mut | Specializes mutable-reference-containing parameters | Does not eliminate all immutable Word type/const/dictionary polymorphism of read_words; not a full-mono replacement |
| --ullbc | Skips structured-body lowering pass | Aeneas consumes structured LLBC; not a backend-compatible bypass |
| Upgrade to known validation fix | Rejects unsupported options earlier | Improves correctness/diagnostics only; no supported monomorphic builtin bridge is established |

No existing supported option was found that both preserves original source,
produces the requested Word-free monomorphic helper, and supplies valid input
to the pinned Aeneas backend for this coercion. A genuine future solution needs
a supported frontend/backend representation path (including correctly declared
monomorphic or deliberately retained polymorphic builtin calls), not an edited
LLBC map or an unverified generic decoder model. Such work is outside this
read-only investigation. The PRNG worker can continue original APIs outside
this blocked seed pipeline; this report does not count the seed as verified.
