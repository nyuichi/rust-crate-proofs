# Original normalize Iterator defaults: finite0..4 character domain

This supplement directly checks actual Rust Iterator::nth/count/last on the
opaque iterator returned by the **unchanged original** normalized function.
It is separate from the existing adapted-source Aeneas claim, original constructor
initialization proof and earlier original finite-input next contract. No Aeneas
mapping/count is added and no char-switch frontend experiment is repeated.

The input is a concrete nonpanicking fused Input iterator over an arbitrary
[char;4], starting at position0 with active length0..4. Every valid Unicode scalar
is included (inactive array entries are arbitrary too). Length is in character
items, not UTF8 bytes. The sole per-harness assumption is active length<=4; no
character alphabet or nth index restriction is imposed. This does not quantify
over arbitrary Iterator implementations, panicking/nonterminating callbacks,
non-fused histories, std::str::Chars internals or arbitrary-length inputs.

An independent eager specification scans original input positions: omit LF
immediately following CR, replace CR by LF, otherwise preserve the character.
It does not call original next or reuse its previous-CR state machine. Empty
input, consecutive CR, CRLF at every position, standalone LF and all non-ASCII
characters occur in the quantified domain.

- nth(index) is checked for **every usize**, including usize::MAX. It returns
  exactly the specification's indexed character or None when out of range.
  After a successful result, all following next calls yield exactly the remaining
  specified suffix; after exhaustion, two further next calls return None.
  Large indices terminate because this concrete input reaches a persistent EOF
  after at most four character items; there is no artificial index/fuel bound.
- count consumes the iterator and returns exactly the specification's output length.
- last consumes it and returns the final specified output or None for empty output.

Three separate sequential harnesses succeed:417+355+390=1162 checks, zero failures,
seven unreachable internal checks per harness. All have explicit unwind8 with
unwinding assertions retained and a60-second timeout; each completed below3 seconds
in the final run. This successful finite-loop unwinding is part of the guarantee,
not a restriction on nth's index. No stubs, private observers, injected original
states, Formatter/model substitutions, source or LLBC transformations are used.

The original package's entire archive/source/manifest is retained byte-identically
in source/. Cargo metadata/lock, compiler version, complete logs and per-harness
metadata are preserved. ./verify.bash archive-checks all files, runs each actual
public method call separately and verifies the final source/results. Kani0.68.0,
CBMC6.11.0, Rustnightly2026-08-21/compiler/std semantics are trusted; other std
versions/configurations/targets are not established. Configuration is
x86_64/default/debug. The original next arbitrary-length Aeneas obstacle remains
unresolved; the other inherited/default/blanket APIs remain independently open.
