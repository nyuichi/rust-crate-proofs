use vstd::prelude::*;

verus! {

/// Production `FastRand::fastrand_n(8)` after `fastrand()` has produced its
/// arbitrary `u32` word.  No distributional property or entropy assumption is
/// needed by watch: the type bound alone makes the high-word reduction a valid
/// BigNotify index.
pub fn fastrand_shard_8(word: u32) -> (shard: usize)
    ensures shard < 8,
    no_unwind
{
    let mul = (word as u64) * 8u64;
    proof {
        vstd::bits::lemma_u64_shr_is_div(mul, 32);
        vstd::arithmetic::power2::lemma2_to64();
        assert(mul < 8 * 4_294_967_296);
        assert(mul as nat / 4_294_967_296 < 8);
    }
    (mul >> 32) as usize
}

/// Minimal body-proved model of the `CONTEXT.rng` Cell orchestration in
/// `thread_rng_n`: use the installed generator when present, otherwise the
/// fallback generator, advance it, put it back, and return the bounded shard.
/// The arbitrary next word and next state represent U02's deterministic RNG
/// step; watch depends only on state replacement and the range theorem above.
pub struct ThreadRngSelector {
    installed_state: Option<u64>,
    last_used_state: Option<u64>,
}

impl ThreadRngSelector {
    pub closed spec fn installed_state(&self) -> Option<u64> {
        self.installed_state
    }
    pub closed spec fn last_used_state(&self) -> Option<u64> {
        self.last_used_state
    }

    pub fn new(installed_state: Option<u64>) -> (result: Self)
        ensures result.installed_state() == installed_state,
            result.last_used_state().is_none(),
        no_unwind
    {
        ThreadRngSelector { installed_state, last_used_state: None }
    }

    pub fn select(
        &mut self,
        fallback_state: u64,
        next_state: u64,
        next_word: u32,
    ) -> (shard: usize)
        ensures
            shard < 8,
            final(self).installed_state() == Some(next_state),
            final(self).last_used_state() == Some(
                if old(self).installed_state().is_some() {
                    old(self).installed_state().unwrap()
                } else {
                    fallback_state
                }),
        no_unwind
    {
        let _current_state = match self.installed_state {
            Some(state) => state,
            None => fallback_state,
        };
        self.last_used_state = Some(_current_state);
        let shard = fastrand_shard_8(next_word);
        self.installed_state = Some(next_state);
        shard
    }
}

pub fn verify_random_and_circular_selectors_share_big_notify_range(
    word: u32,
    ticket: usize,
)
{
    let random = fastrand_shard_8(word);
    let circular = crate::watch_refinement::select_circular_shard(ticket);
    assert(random < 8);
    assert(circular < 8);
}

pub fn verify_thread_rng_fallback_is_reinstalled(
    fallback_state: u64,
    next_state: u64,
    word: u32,
)
{
    let mut selector = ThreadRngSelector::new(None);
    let shard = selector.select(fallback_state, next_state, word);
    assert(shard < 8);
    assert(selector.installed_state() == Some(next_state));
}

} // verus!
