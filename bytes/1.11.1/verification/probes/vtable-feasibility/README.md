# Vtable cycle feasibility probe

This is a diagnostic translation probe for the two `Bytes` clone-vtable cycles
in the real crate. It does not claim a proved runtime contract. Each recorded
run used the repository wrapper with translation only; no Why3 proof was run.
The logs are under `logs/`. Native `cargo check --locked --all-features` passes;
the failures below are translator errors, not Rust type errors or failed VCs.

| Feature | Result | What it establishes |
| --- | --- | --- |
| `static-cycle` | translator reports mutual recursion | The small const-vtable pattern reproduces `from_static -> STATIC_VTABLE -> static_clone -> from_static`. |
| `owned-cycle` | translator reports mutual recursion | The generic associated-const pattern reproduces `from_owned<T> -> Owned<T>::VTABLE -> owned_clone<T> -> from_owned<T>`. |
| `accessor-cycle` | translator rejects the static definition kind | Hiding the table behind an accessor does not establish a route for the actual const table; this probe uses `static`, which vanilla 0.13 does not support. |
| `passed-vtable` | translator reports `unsupported function call type` at the indirect call | Passing the current table alongside the real clone callback arguments would break the source-level lookup cycle, but vanilla 0.13 cannot translate this callback signature/call. It is not a usable source fix. |

The real sites are `Bytes::from_static` and `STATIC_VTABLE` / `static_clone`,
and `Bytes::from_owner` and `Owned<T>::VTABLE` / `owned_clone<T>` in
`src/bytes.rs`. The probe does not model the full `Bytes` object, pointer
ownership, refcounts, or callback behavior, so a successful translation here
would only establish feasibility of this control-flow shape. The callback
parameter experiment is blocked at translation, before any verification
conditions exist.

Current classification: both real sites are translation blockers under vanilla
Creusot 0.13. The passed-table experiment adds an explicit `&'static Vtable`
argument to a callback with the real `AtomicPtr`, byte pointer, and length
arguments, but translation stops at the function-pointer call before VCs are
generated. No runtime edits or trusted contracts were added. Exclude the
vtable/`Bytes` clone path from the current integrated proof target until a
compiler-supported, semantics-preserving restructuring is available.
