# bytes 1.11.1: current target policy

The user changed the target on 2026-10-07 and authorized removal of the
modified `bytes::verified` API and high-rework alternatives identified by the
inventory. Restore the upstream 1.11.1 API and representation while retaining
the genuine Vec reverse-comparison bug fix. This instruction supersedes the
2026-10-06 route-1 target recorded in the historical architecture documents.

Do not claim the retired modified-variant proof gates as current verification.
Keep their archived evidence for historical review. Temporary local trusted
contracts are authorized only with precise assumptions, a reviewed strong
contract, and a concrete path to remove trust with few changes to other code.
They remain open proof obligations; never count assumed bytes-specific
ownership/refcount/last-owner/destructor laws as complete verification. Generic
physical/library TCB remains separately documented. Small trust removal for
actual Clone/automatic Drop is not yet established. Do not remove archival
counterexamples or imply a small tool patch solves these boundaries.

# bytes 1.11.1: architecture decisions

Before changing verification architecture or launching a new proof experiment,
read `verification/ARCHITECTURE_DECISIONS.md` and
`verification/ARCHITECTURE_ASSESSMENT.md` when present. These record user-approved
no-repeat decisions; the user's current instructions take precedence.

Do not retry a frozen method just with new assertions, wrappers, timeouts,
models/agents, or cosmetic source changes. Reopening requires a recorded changed
premise, supporting evidence, a bounded distinguishing experiment, and an update
to the applicable decision. Never discard the previous counterexample.

Until the architecture admission gate passes, do not add disconnected API gates
as progress toward complete verification. Preserve prior component evidence.
Evidence maintenance and genuinely distinguishing architecture experiments are
allowed. A blocked target is not a completed or implicitly reduced target.

Only bytes 1.11.1 is in scope. Commit/push validated increments to
`origin bytes-runtime-verification`; never main or force-push. Existing proof
serialization, elevated Why3 execution, and evidence-audit rules still apply.
