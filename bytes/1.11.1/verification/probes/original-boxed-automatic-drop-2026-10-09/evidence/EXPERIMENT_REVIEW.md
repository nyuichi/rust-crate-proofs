# Interface/tool review

Frontend v1: ordinary Ghost dereference/type syntax; v2: a logic-only raw_pointer
was used in Ghost program context. Restructured clear boundary to borrow the
sealed descriptor as Ghost metadata while deriving the runtime result only from
the actual tagged pointer. No pointer is materialized from a logic result.
These sources and frontend logs are frozen in translation-frontend-v1/v2.

Proof tool parse v1: a generated filename containing a hyphen produced an invalid
Coma span identifier. Renamed boxed-client.rs to boxed_client.rs and retained the
original tasks/log separately; this is not an ownership/body failure.

Body v2, 95/540/8: native callbacks lacked explicit parity/tag-state information,
and the empty constructor's typed contract did not retain Static shape. Review
added locally proved shape postconditions on unchanged new/from_static bodies,
parity-specific callback preconditions and low-bit validity information.

Body v3, 95/530/3: printed null tasks showed the second opaque Bytes-specific
forwarding tag symbol hid its equality to the generic roundtrip input; ordinary
integer mode also leaves bitwise operations opaque. After the second same
interface issue, restructured to one shared generic logical tag symbol and two
body-proved check(ghost)/bitwise_proof mathematical lemmas. No third iteration
of assertion tuning occurred. Body v4 then proves 97/510/0. All failures/positive
diagnostic preserve exact sources/tasks/results/config inputs.

Default native ptr_map uses exposed pointer-to-integer/reconstructed-pointer
semantics, not the miri wrapping-add path. Astra inspected pinned Rust context
and approved this only as an explicit assumed generic exposed-provenance
roundtrip TCB, binding the forward operation to its exact base, with independently
retained full allocation authority required by free. Equal-pointer offset_from
zero needs no additional live-allocation premise in pinned Rust; it grants no
free authority. Pinned source context is retained in native-stdlib.

Independent review found absolute paths in the generated Cargo harness and
fixture that prevented ordinary relocated replay. Native lib/dependency paths
are now exact relative literals; checker validates resolution to the captured
source/production crate. Native MIR and five-case run were regenerated. Archive
restoration layout explicitly reconstructs captured sibling checker/support
imports, without using uncaptured live worktree files.
