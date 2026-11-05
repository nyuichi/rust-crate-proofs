# Four original external API supplements

Any::type_id for Rand32/Rand64 passed39 checks each: all public state/increment
pairs yield TypeId::of<Self>(), differ from the other generator type, and leave
state unchanged. This is not a universal proof of TypeId uniqueness.

CloneToUninit for each type passed42 checks: all state/increment pairs are cloned
from a separate live source into a valid writable aligned MaybeUninit<Self>
stack slot, then public state getters establish source unchanged and destination
exact state. The actual Rust blanket calls the derived Clone/Copy implementation;
those user-excluded derive APIs are not counted as independent new claims.
This fixed-slot specialization does not prove arbitrary heap allocation behavior.

Clone logs warn about unsupported caller_location/foreign panic paths; the
corresponding reachability guards pass SUCCESS. No unsupported path is reached,
no stub/assume or observer was added. Panic/invalid destinations are outside scope.
The built std is nightly08; available docs reviewed were09, not falsely asserted
byte-identical. Four jobs were sequential with60s cap, all finished under1s.
The packaged path dependency is a byte-identical original copy relocated only
after execution. Aeneas machine count increase0; six generic external conversion
families remain conditional/unproved. See RESULTS.json and raw logs.
