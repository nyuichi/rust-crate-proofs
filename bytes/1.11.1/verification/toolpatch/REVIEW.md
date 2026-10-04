# Experimental validator patch

The patch targets official Creusot 0.13.0 commit
318615be3b8bbc60d1f6d52469ba5c0bdebed4f1. It is retained for investigation;
the installed proof compiler and all helper/storage/deallocation proofs use the
unmodified compiler.

Astra reviewed the self-bound exception. It applies only to an impure method's
own trait bound with no direct or external contract, logic, ghost or termination
marker. Associated types, supertraits and mutual bounds retain the original
checks. Astra identified the necessary external-spec guard; that guard was added
before the final candidate build. General soundness has not been established:
implicit invariants, resolve and specification dependencies require further review.

The specification-free positive body and generic caller prove. Logic,
associated-type, ghost, terminating, mutual and contracted cycles still reject;
recursive implementations adding ghost/termination requirements reject. Ordinary
false method bodies translate and fail their VCs. Recorded logs are under
../artifacts/logs/trait-patch-*.log.

Candidate reproduction uses CREUSOT_RUSTC pointing to the patched compiler with
scripts/verify-bytes.sh trait-patch. Do not replace the installed vanilla compiler.
The runtime still encounters the comparison-model ICE with this patch, and
indirect calls, Drop semantics and pointer exposure remain independent blockers.
