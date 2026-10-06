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
