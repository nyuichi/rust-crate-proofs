# Original inherited Clone::clone_from: concrete u64 supplementary verification

For **NoHashHasher<u64> only**, direct Kani execution of the real Rust standard
Clone default body establishes that destination's stored value is replaced by
source's value, and source remains unchanged. A separate prefix-identical source
copy with read-only getters additionally checks the debug written flag. Both
stages succeed independently with std/no_std × debug_assertions true/false.
This is not a generic-all-T proof or verification of the other inherited APIs.

## Reachable-state contract

Source and destination are each independently constructed by original Default,
then either no write or exactly one original write_u64 of any symbolic u64.
This includes all stored u64 values after writing and the initial value0 state;
in debug mode the flag is false initially and true after writing (including
writing zero). No private state is injected. A false flag with arbitrary nonzero
storage, second writes, panicking histories and invalid state fabrication are
not part of the domain.

The harness invokes `<NoHashHasher<u64> as Clone>::clone_from(&mut dst, &src)`.
Its independent expected result is zero if source was not written and the
symbolic source u64 otherwise. Public finish() checks both initial values,
the replaced destination and preserved source. In the observer stage, the
postcondition additionally requires destination/source getter results to equal
that value plus Some(source_written) in debug, or None in release where the
flag field is absent. Destination's previous state can differ independently,
so both true→false and false→true flag replacement are included.

## Two distinct evidence stages

1. `source/` is the exact original release source and manifest from the retained
   archive. `original-stage/` depends on it without any source modification and
   observes only public finish(). Its four Kani logs have47 checks in debug and28
   in release, zero failures in each.
2. `observed-source/` preserves **every original Rust byte as a prefix** and the
   original manifest unchanged, then appends only cfg(kani) read-only getters
   `&NoHashHasher<u64> -> (u64, Option<bool>)`. They project fields, never mutate,
   inject state or alter any original implementation. `tools/verify.py` guards
   original/archive equality, exact prefix+fixed observer suffix and its hash.
   The formal source artifact in source/ remains untouched. `observed-stage/`
   checks value/flag observations in four separate logs:104 checks in debug,
   81 in release, zero failures each. Relating this observation result to the
   original relies on the authenticated unchanged bodies and benign getter
   addition; it is not presented as private-field observation through an
   entirely unmodified public dependency.

All inputs are quantified symbolically; the harnesses have no assumptions,
stubs, models, warning suppression, custom unwind bounds or disabled checks.
The real compiler-provided default Clone::clone_from and original manual clone
are executed. `evidence/original-goto-functions.txt` and the extracted
`actual-std-clone-body.txt` retain compiler GOTO evidence: core/src/clone.rs249
calls the original manual clone and assigns its result to destination. Kani
metadata records no stub attributes. These are Rust/Kani results; the earlier
Lean theorem about official Clone's default **model composition** is separate
and is not used to establish this new body-execution evidence.

## Configurations and reproduction

`./verify.bash` runs all eight small jobs sequentially and archive-checks source
before/after. Each resolved metadata graph records dependency features ['std']
or [] explicitly (dependency default features are always disabled). Root
compile-time guards require the intended debug_assertions mode. The observer
postcondition independently distinguishes Some(flag) from None, detecting a
mode mismatch in the observed dependency. Harnesses remain ordinary std users;
no_std describes the original dependency's feature configuration.

All logs, metadata and results retain separate stage/configuration identities.
Kani0.68.0, its Rust nightly2026-08-21 compiler/standard library, CBMC6.11.0 and
supported Rust semantics are trusted. This does not independently verify std's
Clone default for every Rust type, prove pointer/layout behavior, or establish
other targets/std versions. No new independent published API claim is created
by this packet, and the existing fourteen Aeneas mappings remain unchanged.
