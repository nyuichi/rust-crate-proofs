# Owned-region kernel probe

This probe checks the pure resource-algebra ledger for half-open slot intervals. The positive run constructs a model ledger, splits it in a helper, returns both halves, and rejoins them in the caller. Contracts preserve the resource identity, interval domains, known values, and an `Unknown` slot.

The ledger is algebra-only. The model constructor does not connect a resource to an allocation, pointer, or physical byte, and the slot model does not authorize reads, writes, initialization claims, or deallocation. A separately reviewed physical bridge is required for those claims.

Run the positive probe from the `bytes/1.11.1` directory with:

```sh
./scripts/verify-bytes.sh owned-region-kernel
```

The captured wrapper output is in `logs/positive.log`; it ends with `Proved (7 files)`. The expected overlap rejection is run with:

```sh
./scripts/verify-bytes.sh owned-region-kernel --features wrong_overlap
```

That run must fail at `Coma.vc_rejected_overlap` because the probe supplies an overlap premise while `join` requires contiguous intervals. Its captured output is in `logs/negative-overlap.log`. This negative VC checks the join precondition only; it does not construct a physical alias or establish a runtime ownership property.
