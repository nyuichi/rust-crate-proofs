# Sequential retirement registry

Vanilla Creusot/creusot-std 0.13. This is a uniquely coordinated sequential
proof over actual B1 physical resources, not native Shared or atomic refcounts.

The private coordinator owns Recovery, PhysicalPool, and returned registration
resources inside NonAtomicInvariant. Two private affine tickets carry separate
components of one exclusive registration RA. Packet contracts tie each ticket
to the allocation, capacity and exact region. open_mut preserves allocation and
registration identity while publishing pending-bit progress; retirement consumes
both the ticket and physical region. Finalization consumes the coordinator and
requires both registrations returned. Native B3 runs unconditionally outside
ghost code; a ghost flag never chooses whether to deallocate.

The default proof passes 27 files. Two native tests cover both retirement orders,
all split positions of a nonempty buffer, zero capacity, empty reserved capacity,
and spare capacity. No new trusted declaration or core/library edit was added.

Run the proof wrapper with elevated Why3 execution:

- `./scripts/verify-bytes.sh sequential-retirement-registry`
- `./scripts/verify-bytes.sh sequential-retirement-registry --features negative_missing_empty_ticket`

The negative emits 28 files and intentionally fails only
`Coma.vc_reject_missing_empty_ticket` at one pending-ticket guard. It splits at
zero, omits the empty left packet, and retires the full right region. Complete
byte coverage alone cannot authorize finalization. Do not execute this invalid
feature path as a native test.

Final exact sources, configuration, wrapper, logs, Coma and proof JSON are under
`../../artifacts/evidence/sequential-retirement-registry/`. The unique mutable
coordinator exposes progress unavailable through fixed-public shared opening;
it does not prove independently shared handles, native reference counts,
control-block destruction, concurrency or automatic Drop.
