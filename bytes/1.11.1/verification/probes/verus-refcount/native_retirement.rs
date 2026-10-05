#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
use vstd::prelude::*;
use verus_state_machines_macros::tokenized_state_machine;

verus! {
tokenized_state_machine!(Retirement<R> {
    fields {
        #[sharding(constant)] pub expected: Map<nat, R>,
        #[sharding(variable)] pub remaining: nat,
        #[sharding(variable)] pub done: bool,
        #[sharding(map)] pub pending: Map<nat, R>,
        #[sharding(storage_map)] pub retired: Map<nat, R>,
    }
    #[invariant]
    pub fn count_agrees(self) -> bool { self.remaining == self.pending.len() }
    #[invariant]
    pub fn partition(self) -> bool {
        &&& self.pending.dom().disjoint(self.retired.dom())
        &&& if self.done {
            self.pending == Map::empty() && self.retired == Map::empty()
        } else {
            self.pending.union_prefer_right(self.retired) == self.expected
        }
    }
    init! { initialize(expected: Map<nat, R>) {
        init expected = expected;
        init remaining = expected.len();
        init done = false;
        init pending = expected;
        init retired = Map::empty();
    } }
    transition! { retire(id: nat, r: R) {
        require(!pre.done);
        remove pending -= [id => r];
        deposit retired += [id => r];
        update remaining = (pre.remaining - 1) as nat;
    } }
    transition! { finish() {
        require(!pre.done);
        require(pre.remaining == 0);
        withdraw retired -= (pre.expected);
        update done = true;
    } }
    #[inductive(initialize)] fn initialize_inductive(post: Self, expected: Map<nat,R>) {
        assert(post.pending.union_prefer_right(post.retired) =~= expected);
    }
    #[inductive(retire)] fn retire_inductive(pre: Self, post: Self, id: nat, r: R) {
        assert(post.pending.union_prefer_right(post.retired) =~= pre.expected);
    }
    #[inductive(finish)] fn finish_inductive(pre: Self, post: Self) {
        assert(pre.pending =~= Map::<nat,R>::empty());
        assert(pre.retired =~= pre.expected);
        assert(post.retired =~= Map::<nat,R>::empty());
    }
});
}
fn main() {}
