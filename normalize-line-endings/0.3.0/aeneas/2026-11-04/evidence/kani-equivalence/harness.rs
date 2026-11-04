//! Relational proof of the three character-pattern-to-guard edits.
//!
//! The actual source files are mechanically embedded by prepare.py. The only
//! added function exposes private fields before and after one call to next.

#[path = "generated/original.rs"]
mod original;
#[path = "generated/adapted.rs"]
mod adapted;

/// A coupled trace of up to two successful callback invocations. The returned
/// values and post-callback observable state are independent symbolic inputs.
/// None followed by Some is permitted, so this model does not require fusion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ScriptedIterator {
    results: [Option<char>; 2],
    post_states: [u64; 2],
    observable_state: u64,
    calls: u8,
}

impl Iterator for ScriptedIterator {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        match self.calls {
            0 => {
                self.calls = 1;
                self.observable_state = self.post_states[0];
                self.results[0]
            }
            1 => {
                self.calls = 2;
                self.observable_state = self.post_states[1];
                self.results[1]
            }
            _ => panic!("adapter consumed more than two callback results"),
        }
    }
}

#[kani::proof]
fn one_step_equivalence() {
    let prev: bool = kani::any();
    let iter = ScriptedIterator {
        results: [kani::any(), kani::any()],
        post_states: [kani::any(), kani::any()],
        observable_state: kani::any(),
        calls: 0,
    };
    let (original_result, original_prev, original_rest) =
        original::verification_probe_step(iter, prev);
    let (adapted_result, adapted_prev, adapted_rest) =
        adapted::verification_probe_step(iter, prev);

    assert_eq!(original_result, adapted_result, "returned Option<char>");
    assert_eq!(original_prev, adapted_prev, "next prev_was_cr state");
    assert_eq!(original_rest.calls, adapted_rest.calls, "number of callback calls");
    assert_eq!(original_rest.observable_state, adapted_rest.observable_state,
        "post-callback iterator state");
    assert_eq!(original_rest, adapted_rest, "complete remaining scripted iterator");
    assert!(original_rest.calls >= 1 && original_rest.calls <= 2);
}
