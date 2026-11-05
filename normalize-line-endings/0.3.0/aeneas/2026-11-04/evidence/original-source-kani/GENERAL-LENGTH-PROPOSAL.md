# Unimplemented general-length original-source routes

The new original-body Kani result is length 0..4, not an induction theorem.
Increasing the array/unwind bound only increases that finite bound.

One prospective route is an inductive one-step original-iterator contract:
record the previous-CR flag and a pure input suffix, relate one original next
call (which consumes at most two input chars) to a list normalization step,
then prove a simulation/termination theorem for arbitrary finite lists. The
current original/adapted two-reply harness checks one-step behavior but is not
itself that full inductive theorem or arbitrary-iterator contextual equivalence.
Kani loop contracts would require checked loop invariants, faithful iterator
state, and separate support evaluation; none were implemented or verified here.

A second route is an officially supported char-switch lowering, or a verified
LLBC transformation preserving branch selection/call order and state. It could
connect original next to the existing arbitrary finite-list Lean result, but
requires a proof of the transformation and exact generated-callgraph guards.
The earlier unverified LLBC normalization experiment does not discharge that
obligation. The constructor-only original extraction avoids next entirely and
therefore does not establish this bridge.
