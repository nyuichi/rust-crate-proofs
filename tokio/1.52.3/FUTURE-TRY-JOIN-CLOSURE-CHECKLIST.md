# Tokio F01 future combinator closure checklist

Scope: Tokio 1.52.3 `future::maybe_done`, private process `try_join3`, and the
public `try_join!` macro under `macros`.

| Obligation | Evidence | Status |
|---|---|---|
| MaybeDone state machine | Future/Pending, Future/Ready→Done, stable Done repoll, Gone repoll panic | body-proved / tested |
| output access | `output_mut` is available exactly in Done and preserves unique output ownership | body-proved / tested |
| output take | Done→Gone transfers the exact output once; Future/Gone returns None | body-proved / tested |
| cancellation/drop | Future, completed output, and Gone resources remain disjoint and are released exactly once | body-proved / tested |
| private try_join3 | fixed left-to-right polling, stable completed branches, exact ordered tuple | body-proved / process cfg-compiled |
| early error | first observed error is returned; later branches are not polled and all retained futures/outputs are dropped | body-proved / tested |
| terminal repoll | successful or error completion leaves a Gone branch and a later poll takes the panic path | body-proved / tested |
| normal rotation | three-branch cyclic order is 0-1-2, 1-2-0, 2-0-1 and wraps exactly | body-proved / tested |
| biased order | every poll begins at branch zero | body-proved / tested |
| macro normalization/arities | empty, one, two, three, and recursive eight-branch expansions preserve declaration-order tuple output | compile/runtime-tested |
| fairness connection | each normal poll advances the starting branch; biased mode retains declaration order | body-proved for canonical three branches / runtime-tested |
| representation | tuple futures remain inline and pinned; existing size regressions remain exact | compile/runtime-tested |

Pin projection, `Context`/Waker mechanics, arbitrary inner Future execution,
and arbitrary output/error Drop behavior remain the frozen generic-language
foundation. Tokio-owned state selection, resource transfer, poll ordering, and
early-return cleanup are closed above it.

No production logic, temporary trusted function, `external_body`, `assume`,
`admit`, or new axiom was introduced.
