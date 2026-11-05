# Original normalize Iterator character search — bounded input, full target domain

This supplement verifies **position and find only**, directly calling the actual
Rust Iterator defaults on the opaque iterator returned by the unchanged original
normalized API. It extends neither the existing adapted1 Aeneas mapping nor the
separate original nth/count/last packet into an arbitrary-iterator theorem.

The concrete fused Input iterator starts at position0 over an arbitrary [char;4]
with active length0..4. Every valid Unicode scalar (and arbitrary inactive entry)
is included. A separately symbolic target is also every valid Unicode char,
with no alphabet/target restriction. The sole assumption in each harness is
active length<=4; this is a bound in char items, not UTF8 byte length. The
predicate family is exactly pure scalar equality to that target. Arbitrary
FnMut callbacks, side effects, panics or divergence are not claimed.

An independent eager CR/LF specification computes the normalized sequence by
replacing CR with LF and dropping LF immediately after CR in original input
positions. It does not call original next or copy its previous-CR machine.
The specification scans its output for the first character equal to target.

- position returns that first index in **normalized character items**, or None.
  This is not an index in original input/UTF8 bytes.
- find returns the corresponding first normalized character, or None.
- For either method, if found, later next calls produce exactly the suffix
  after that match, then persistent EOF (two additional None observations).
  If absent, the iterator is exhausted and next returns None twice.

The consumption claim is the observable normalized-output remainder, not the
private raw input cursor or a callback invocation count. For example, searching
for LF in a sequence beginning CR,LF,x matches the normalized LF and leaves x
as the next normalized character. How the underlying CRLF is staged privately
is deliberately unobserved. All CR/LF boundaries/repetitions, empty inputs,
non-ASCII targets and target CR (which cannot occur in normalized output) are
covered by the quantified domains, rather than selected examples.

Two independent final jobs pass480 and429 checks (909 total), zero failures,
seven unreachable checks each, with explicit unwind8 and unwinding checks kept
on. Both completed below5 seconds, with a60-second per-harness timeout. There
are no stubs, original private-state observers/injections, source/LLBC edits,
new standard-library models or disabled checks. Entire original archive/source/
manifest bytes, actual public calls, metadata/lock/compiler and logs are saved.

Configuration: x86_64/default/debug; Kani0.68.0, CBMC6.11.0, Rustnightly2026-08-21
and supported compiler/std semantics are trusted. Arbitrary lengths, arbitrary I,
non-fused histories, other predicates/options/configurations/std versions are
unproved. Other inherited APIs remain independent obligations. This is not an
Aeneas solution to the original char-switch obstacle, which was not retried.

inventory-reconciliation-notes.json retains all six inventoried methods with
unsatisfied extra Self/Item bounds separately: they do not count as callable
guarantees. In particular original return type does not implement/expose
DoubleEndedIterator; nth_back was not forced onto an artificial adapter, and
rposition's extra bounds are not met. min/max/collect/fold/for_each were not added
merely to increase API counts. The earlier position-only probe is historical;
final two logs establish this frozen two-method packet.

./verify.bash validates all original archive files and runs each harness separately.
Publication integration is performed by the coordinating publisher, not this worker.
