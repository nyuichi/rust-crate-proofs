# Saved external16 closure obligations

Exact16.json preserves the default rustdoc ledger rows. For each Rand32/Rand64,
concrete From<Self>, Borrow<Self>, BorrowMut<Self>, Any::type_id and unstable
CloneToUninit::clone_to_uninit account for10. Into<U>, TryFrom<U>, TryInto<U>
account for6 conditional families. ToOwned is absent from this saved ledger.
The14 std-derived rows are a different user-excluded group. External downstream
trait implementations remain an open universe, not claimed enumerated here.

Prepared two original-path-dependency Kani jobs quantify every public from_state
state/increment pair and every replacement pair. They check From identity,
Borrow observations and BorrowMut mutation reflection via the original public
state getter. No private injection, observer or model is used. No solver result
yet. Cargo check was only a lightweight ordinary compilation; cfg(kani) proof
bodies still require Kani frontend checking.

Conditional followup should extract original core conversion blanket bodies with
Aeneas: Into delegates U::from(self), TryInto delegates U::try_from(self), and
TryFrom<U> under U:Into<Self> returns Ok(U::into(value)). Supply arbitrary modeled
trait dictionaries and prove exact result/effect forwarding, retaining Rust
bounds. This is not proof that arbitrary user callbacks are correct/terminating.
An identity U=Self Kani instantiation alone does not close these generic families.

Any::type_id needs explicit compilerintrinsic/TypeId semantic boundary. Comparing
TypeIds alone is insufficient to prove universal uniqueness or type semantics.
CloneToUninit needs valid allocated/aligned destination and initialization/drop
conditions; no unsafe-destination experiment has been prepared or executed.
Neither is completed. Original standard-library body/version, concrete calls,
configuration and extraction feasibility must be recorded before full closure.
