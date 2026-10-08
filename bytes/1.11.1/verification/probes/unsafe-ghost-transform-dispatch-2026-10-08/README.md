# Checked ghost transformation inside an erased unsafe callback

This probe extends scalar unsafe registration with an affine transformation
inside the checked shim. It does not verify any Bytes ownership law, native
vtable field selection, arbitrary ghost-instrumented code, or public Clone.

The private `declare_transform!` macro has a closed generation skeleton:

- One checked native unsafe scalar function computes `x + step`.
- The checked shim receives `(x, Ghost<Resource<Excl<Int>>>)`, invokes that same
  native function exactly once, and transforms the resource inside `ghost!`.
- The ghost block consumes the actual input resource and uses the shipped
  `ExclUpdate` to set its value to `Excl(native_result@)`. Its resource ID remains
  unchanged. The output is not a freshly allocated substitute.
- The shim returns precisely the native scalar result and the transformed ghost
  resource. All extra execution is in checked ghost code, which may not write
  the native result and must terminate successfully.

Both unsafe native functions and both shims are body proved. The final clients
consume the actual affine input and establish exact native value, output resource
value and resource ID through the shim's checked postcondition. A separate client
allocates the initial exclusive resource and calls the registered transformation.

## Explicit generic TCB

`registered_erasure(native, shim)` means this checked shim's runtime erasure is
the specified native pointer invocation, with ghost input/output accounted for
by its checked ghost steps. The trusted registration getter emits the native
pointer and a ghost-only safe shim pointer, equates its FnExt contract to the
same checked shim, and establishes that relation. The macro uses the same native
identifier in the getter and in the shim's sole runtime call. It is private;
there is no arbitrary pointer/shim registration factory.

`invoke_transform` is the generic trusted invocation boundary. It requires the
matching erasure certificate and shim precondition, consumes the ghost input,
and ensures exactly the shim postcondition. Native execution calls only the
unsafe native pointer, once. It returns an erased `Ghost` marker; it does not
execute a runtime resource update or call the shim. Logical ghost execution is
justified by the checked shim under the explicit erasure rule, not by treating
that marker as a newly allocated resource.

This rule assumes safety and normal-return contract correspondence. It has no
termination claim for arbitrary native calls and no unwind guarantee. No code
address equality/injectivity is assumed. Corrupting the trusted getter/invoker
is a TCB change requiring audit; the controls do not prove these implementations.

The analogue is shipped FnExt call contracts plus the checked ghost mechanism
and Resource/ExclUpdate rules. Exact source snapshots include ghost purity and
termination checks. Future generic function reification and verified erasure
support could replace this boundary while retaining the checked clients; no
small backend implementation or existing general mechanism is claimed.

## Controls and captures

`./run-proof.sh` runs the default gate under the shared lock, one prover and
1024 MiB; no SC feature or solver autodetection. Native: `cargo test --locked`.

Feature-gated controls are:

- `negative_wrong_certificate`: identity native pointer with successor shim;
  only matching registered-erasure admission should fail.
- `negative_duplicate`: reuse input resource after the callback consumes it;
  expected frontend E0382.
- `negative_ghost_write`: write the scalar native result inside the shim's
  ghost block; expected ghost-erasure purity rejection.

Archives and their manifests record actual outcomes, not just these expectations.
A final default capture follows the controls. `capture.py` checks complete engine
status and Coma/JSON correspondence, and snapshots exact source, Std analogues,
frontend restrictions and logs.

## Remaining integration constraint

This skeleton contains one scalar native call followed by ghost-only updates.
An actual Bytes clone callback performs a native atomic operation whose
Committer/state transition belongs at that event. This probe does not permit
replaying that atomic call, transferring a Committer from a different event, or
assuming the ownership transition merely because a native function returns.
The checked bytes callback and its actual event instrumentation must have the
same structurally certified erasure. Quota-free registration, arbitrary release
interleavings, vtable selection, and automatic Drop remain separate obligations.

## Completed results

The final default replay completes with `Proved (6 files)`, 6 Coma/6 proof JSON,
zero nulls and exit 0. The native test passes four scalar inputs. No source edits
were made between initial positive, controls and final default replay.

The wrong-certificate gate completes with 7 Coma/7 proof JSON and exactly one
null (exit 1). The installed Why3 API's print-only export confirms its exact goal
is `registered_erasure_u32(identity_native_pointer, successor_shim_pointer)`.
The callback precondition and requested result/resource postconditions otherwise
prove. Duplication is E0382 on the consumed `input`; the ghost-write control is
`cannot write to a non-ghost variable in a ghost! block` at the native scalar.
Both frontend controls generate zero Coma/VC files. Their immutable archives
remain separate from the restored positive source and final proof tree.
