# Baseline rand_core0.10.0 variant (separate executions)

This variant pins the dependency exactly to0.10.0 to match PCG25/report29.
The original0.10.1 packet remains unchanged in ../kani-pcg-debug; its results
are not substituted for this variant. All four new baseline executions completed successfully; their independent logs
and results are retained here. Source/harness are identical to the separate0.10.1
packet, but dependency locks/metadata and verification executions are distinct.

# PCG hand-written Debug: constructor-input constant-output supplement

The four original rand_pcg0.10.2 manual Debug bodies all ignore self and write a
fixed redacted type-name literal. Their default/serde inventory entries were
unproved. This packet verifies useful **fixed-writer/default-format** contracts
through unchanged original public constructors, without private injection,
observers or a new Formatter model. It adds zero Aeneas/machine API claims.
Oorandom11.1.5 has no manual Debug: Rand32/Rand64 use derive(Debug), so those
standard-derived implementations remain excluded under the user's instruction.

## Precisely quantified domain

| Actual Rust type | Inherent constructor inputs | Seed constructors | Exact output |
|---|---|---|---|
| Lcg64Xsh32 | new and from_state:every u64 state/stream pair | from_seed:every16-byte array; seed_from_u64:every u64 | `Lcg64Xsh32 {}` |
| Lcg128Xsl64 | new and from_state:every u128 state/stream pair | every32-byte seed; every u64 seed | `Lcg128Xsl64 {}` |
| Lcg128CmDxsm64 | new and from_state:every u128 state/stream pair | every32-byte seed; every u64 seed | `Lcg128CmDxsm64 {}` |
| Mcg128Xsl64 | new:every u128 state | every16-byte seed; every u64 seed | `Mcg128Xsl64 {}` |

The SeedableRng methods execute the actual rand_core0.10.0 helpers and original
PCG constructors; no seed decoding model is substituted. Constructor selection
is a valid finite Rust enum. Inputs are symbolic full-width integers/arrays;
there are no user assumptions restricting seeds, values or writer bool. These
are all inputs of the **listed** constructors, not arbitrary callback histories
for inherited from_rng/try_from_rng or serde deserialization. Existing aliases
such as Pcg32 are alternate names, not additional independent Debug bodies.

Each constructed RNG is formatted by ordinary `write!(&mut sink,"{:?}",rng)`.
The writer has an initialized fixed64-byte buffer and one symbolic mode:
accept every write, or fail on its first attempted write. Successful formatting
must emit exactly the listed literal, return Ok, have exactly the literal length
and leave unused bytes zero. This universal fixed output across all selected
constructor inputs is a scoped redaction/state-independence property: it reveals
no state/seed digits in this output domain. Error mode must attempt a nonempty
write, return Err and leave the length and all64 bytes at zero. Expected bytes
are independent static specifications, not output obtained by another formatter.

## Evidence and limits

Four harnesses succeeded, sequentially, each below7 seconds with an enforced
60-second harness timeout:2233+2239+2239+2214 =8925 checks, zero failures and133
unreachable checks per harness. Explicit unwind80 keeps unwinding assertions
active; full buffer comparison is indexed, not a dynamic-length memcmp. No
stubs, disabled checks, panic suppression, private states or custom Formatter
semantics are used. Unsupported panic/caller-location inventory entries are
not silently accepted: the corresponding reachability guards succeed.

This proves only default {:?} into this concrete accepting/first-error writer.
Arbitrary Formatter options/writers, partial-prefix write failures, short
capacity, other targets/profiles, serde, RNG callback constructors, post-RNG
mutation histories, RNG distribution/seed quality and generic formatting
noninterference are outside this result. The body inspection explains why a
broader constant-display property is plausible; it is not promoted to an
unrestricted Formatter theorem. Existing Aeneas counts are unchanged.

The wrapper depends directly on ../sources/rand_pcg-0.10.2; it does not copy or
edit original source/manifest. tools/check-source.py compares them against the
retained release archive and recorded hashes. Keep that source/archive layout
when reproducing, or relocate dependency paths explicitly. Cargo metadata pins
rand_pcg features=[] and rand_core0.10.0. Kani0.68.0/CBMC6.11.0 and Rust nightly
2026-08-21 with their compiler/standard-library semantics are trusted.

`./verify.bash` rechecks source, runs each harness separately with a60-second
engine timeout, then validates results/source again. Per-harness logs and
api-bindings.json retain canonical identities and source spans. No server call,
publication or mutation of an existing proof artifact is performed here.
