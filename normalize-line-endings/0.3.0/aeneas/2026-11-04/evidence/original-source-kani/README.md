# Original normalize-line-endings 0.3.0 supplement

This packet proves original constructor initialization with Aeneas and checks
original iterator behavior for **finite inputs of length 0 through 4 only** with
Kani. It does not prove the original iterator for arbitrary-length inputs.
The upstream Rust source and manifest are unchanged and archive-checked.
This is separate from the existing adapted-source finite-list Lean development
and its original/adapted one-step equivalence experiment.

`./verify.bash` re-extracts only the original constructor, compares generated
Lean, builds/audits its theorem, then runs the original-source Kani harness.
The environment paths in this script are specific to the recorded workspace.

## Original constructor

Official Charon `--start-from crate::normalized` followed by the pinned official
Aeneas backend succeeds without changing source, LLBC or generated Lean.
`OriginalConstructor.initializes` quantifies over every translated iterator
value and iterator dictionary: the constructor returns that same iterator and
sets its private previous-CR flag to false. It neither calls `next` nor proves
subsequent output behavior. The dictionary is the ordinary official Iterator
model, not a new custom opaque model. Audit uses only `propext` and `Quot.sound`.

## Direct original-source Kani behavior (length 0..4)

`src/lib.rs` depends directly on the archive-identical release in `source/`.
Its initialized, nonpanicking, finite `Input` iterator stores four arbitrary
Rust `char` values and a symbolic active length constrained to 0..4. This is
all valid Unicode scalar values, including non-ASCII; inactive entries are
arbitrary too. The single assumption specifies the intended finite domain.
No character alphabet restriction, stubs, or disabled checks are used.

The independent eager specification scans the active input once: omit LF
exactly when its immediately preceding input character is CR; otherwise emit
LF for CR and emit the original character for everything else. Empty input,
consecutive CR, CRLF boundaries and repetitions, standalone LF, and arbitrary
Unicode are included by the quantified domain, rather than selected examples.

For every input in that domain, actual original `normalized(...).next()`:

- emits exactly the specification's sequence, followed by `None` twice;
- never emits CR and emits no more characters than the active input length;
- returns the same sequence when that output is normalized a second time;
- initially reports `size_hint() == (0, None)`.

The `size_hint` result is the inherited **standard Iterator default** reached
through the returned opaque iterator; the crate has no hand-written override.
The two `None`s establish persistence only for this concrete exhausted finite
input iterator; they do not assert the wrapper implements `FusedIterator` for
all iterator inputs. Side effects, panicking/nonterminating iterators,
non-fused input histories, arbitrary-length streams, formatting and allocation
behavior are outside this contract. No UTF-8 byte encoding claim is made: the
API consumes chars, not bytes.

The harness has an explicit unwind bound of 8, with unwinding assertions left
enabled. All relevant finite loops are at most four iterations; the successful
unwinding checks are part of the result. An initial whole-array equality used
a compiler memcmp loop exceeding this bound; replacing only that assertion by
four indexed char comparisons retained exactly the same equality contract.
The initial failure is preserved, not counted as a verification success.

## Official backend investigation and remaining blocker

The existing MIR built/promoted/elaborated/optimized and
MatchBranchSimplification attempts all retain char switches (prior evidence
in the original artifact's `evidence/original-mir-options/`). Here, official
`--monomorphize --start-from crate::scalar_first --include normalize_line_endings`
reaches the unchanged dependency `Iterator::next` with a simple `Option<char>`
iterator, avoiding array-iterator frontend issues. Charon succeeds; Aeneas
still fails with `Unsupported switch scrutinee` at original source lines43-66.
`investigation/scalar-mono/` preserves both tool logs and the original LLBC.
The earlier array caller independently hit an array-expansion issue too and
is preserved as `investigation/mono-*`, not used to infer the scalar result.

Pinned Aeneas `src/interp/InterpStatements.ml:870-975` handles boolean/integer
switch values but falls through for symbolic char. CLI help and rustc option
inventories are preserved. No supported char-switch lowering configuration was
found. This is a finding about the checked pinned pipeline, not a claim that
all future frontend/compiler configurations are impossible. The packet does
not use the earlier unverified LLBC rewriting experiment or adapted Rust.

`results.json` and `api-bindings.json` keep original-constructor Aeneas and
bounded-original-iterator Kani statuses distinct. Publication is handled by
the coordinating agent, not by this packet.
