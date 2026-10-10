#![allow(unexpected_cfgs)]

// Translation-only reproduction of the actual httparse runtime-dispatch body.
// The feature split lets the static blocker be isolated from atomic access.

#[cfg(feature = "static-cache")]
mod static_cache {
    use creusot_std::prelude::*;
    use std::sync::atomic::{AtomicU8, Ordering};

    const AVX2: u8 = 1;
    const SSE42: u8 = 2;
    const NOP: u8 = 3;

    fn detect_runtime_feature() -> u8 {
        if is_x86_feature_detected!("avx2") {
            AVX2
        } else if is_x86_feature_detected!("sse4.2") {
            SSE42
        } else {
            NOP
        }
    }

    static RUNTIME_FEATURE: AtomicU8 = AtomicU8::new(0);
    static OTHER_FEATURE: AtomicU8 = AtomicU8::new(0);

    #[ensures(result@ == 1 || result@ == 2 || result@ == 3)]
    pub fn get_runtime_feature() -> u8 {
        let mut feature = RUNTIME_FEATURE.load(Ordering::Relaxed);
        if feature == 0 {
            feature = detect_runtime_feature();
            RUNTIME_FEATURE.store(feature, Ordering::Relaxed);
        }
        feature
    }

    // These independent accessors make the required stable distinction between
    // two statics observable in a future static-reference encoding.
    pub fn load_other() -> u8 {
        OTHER_FEATURE.load(Ordering::Relaxed)
    }
}

#[cfg(feature = "parameterized-cache")]
mod parameterized_cache {
    use creusot_std::prelude::*;
    use std::sync::atomic::{AtomicU8, Ordering};

    const AVX2: u8 = 1;
    const SSE42: u8 = 2;
    const NOP: u8 = 3;

    // Parameterized runtime detector boundary. This isolates the exact
    // load/detect/store sequence while preserving the runtime atomic calls.
    // The parameter stands for the concrete detector result (AVX2/SSE42/NOP).
    #[requires(detected@ == 1 || detected@ == 2 || detected@ == 3)]
    #[ensures(result@ == 1 || result@ == 2 || result@ == 3)]
    pub fn get_runtime_feature(cache: &AtomicU8, detected: u8) -> u8 {
        let mut feature = cache.load(Ordering::Relaxed);
        if feature == 0 {
            feature = detected;
            cache.store(feature, Ordering::Relaxed);
        }
        feature
    }

    pub fn read_same(cache: &AtomicU8) -> (u8, u8) {
        (
            cache.load(Ordering::Relaxed),
            cache.load(Ordering::Relaxed),
        )
    }

    pub fn read_distinct(left: &AtomicU8, right: &AtomicU8) -> (u8, u8) {
        (
            left.load(Ordering::Relaxed),
            right.load(Ordering::Relaxed),
        )
    }

    const _: u8 = AVX2;
    const _: u8 = SSE42;
    const _: u8 = NOP;
}

#[cfg(feature = "model-wrapper")]
mod model_wrapper {
    use creusot_std::{
        logic::FMap,
        prelude::*,
        std::sync::{
            atomic::{AtomicU8, Ordering},
            committer::Committer,
        },
    };

    // Isolates the creusot-std relaxed atomic contract. This wrapper newtype
    // is proof-only until an erasure/runtime bridge to the upstream static is
    // established.
    pub fn load(cache: &AtomicU8) -> u8 {
        cache.load::<_, Ordering::Relaxed>(ghost!(
            |_committer: &Committer<AtomicU8, u8, Ordering::Relaxed, Ordering::None>| {}
        ))
    }
}

#[cfg(feature = "model-dispatch")]
pub mod model_dispatch {
    use creusot_std::{
        ghost::{perm::Perm, Ghost},
        logic::FMap,
        prelude::*,
        std::sync::{
            atomic::{AtomicU8, Ordering},
            committer::Committer,
            view::{ReleaseSyncView, SyncView, Timestamp},
        },
    };

    pub const AVX2: u8 = 1;
    pub const SSE42: u8 = 2;
    pub const NOP: u8 = 3;

    #[logic(open, inline)]
    pub fn supported_feature_value(value: u8, cpu_avx2: bool, cpu_sse42: bool) -> bool {
        pearlite! {
            value@ == NOP@
                || (value@ == AVX2@ && cpu_avx2)
                || (value@ == SSE42@ && cpu_sse42)
        }
    }

    #[logic(open, inline)]
    pub fn cache_history_invariant(
        history: FMap<Timestamp, (u8, SyncView)>,
        cpu_avx2: bool,
        cpu_sse42: bool,
    ) -> bool {
        pearlite! {
            forall<t: Timestamp>
                history.get(t) == None
                    || history.lookup(t).0@ == 0
                    || supported_feature_value(history.lookup(t).0, cpu_avx2, cpu_sse42)
        }
    }

    // This has the runtime dispatch's load / AVX2 / SSE4.2 / NOP / store shape.
    // The booleans stand for stable CPU capability facts. `ownership` is the
    // existing AtomicU8-history resource; static construction is separate.
    #[requires((*ownership).ward() == cache)]
    #[requires(cache_history_invariant(
        *(*ownership).val(), cpu_avx2, cpu_sse42
    ))]
    #[ensures(supported_feature_value(result, cpu_avx2, cpu_sse42))]
    #[ensures(cache_history_invariant(
        *((^ownership).val()), cpu_avx2, cpu_sse42
    ))]
    pub fn get_runtime_feature(
        cache: &AtomicU8,
        cpu_avx2: bool,
        cpu_sse42: bool,
        mut ownership: Ghost<&mut Perm<AtomicU8>>,
        mut sync_view: Ghost<&mut SyncView>,
        release_view: Ghost<ReleaseSyncView>,
    ) -> u8 {
        let mut feature = cache.load::<_, Ordering::Relaxed>(ghost!(
            |committer: &Committer<AtomicU8, u8, Ordering::Relaxed, Ordering::None>| {
                committer.shoot_load(&*ownership, &mut **sync_view);
            }
        ));
        if feature == 0 {
            feature = if cpu_avx2 {
                AVX2
            } else if cpu_sse42 {
                SSE42
            } else {
                NOP
            };
            cache.store::<_, Ordering::Relaxed>(
                feature,
                ghost!(
                    |committer: &mut Committer<
                        AtomicU8,
                        u8,
                        Ordering::None,
                        Ordering::Relaxed,
                    >| {
                        committer.shoot_store(
                            &mut **ownership,
                            &mut **sync_view,
                            *release_view,
                        );
                    }
                ),
            );
        }
        feature
    }

    #[cfg(feature = "negative-store")]
    #[requires((*ownership).ward() == cache)]
    #[requires(cache_history_invariant(
        *(*ownership).val(), cpu_avx2, cpu_sse42
    ))]
    #[ensures(cache_history_invariant(
        *((^ownership).val()), cpu_avx2, cpu_sse42
    ))]
    pub fn malformed_store(
        cache: &AtomicU8,
        cpu_avx2: bool,
        cpu_sse42: bool,
        mut ownership: Ghost<&mut Perm<AtomicU8>>,
        mut sync_view: Ghost<&mut SyncView>,
        release_view: Ghost<ReleaseSyncView>,
    ) {
        cache.store::<_, Ordering::Relaxed>(
            4,
            ghost!(
                |committer: &mut Committer<
                    AtomicU8,
                    u8,
                    Ordering::None,
                    Ordering::Relaxed,
                >| {
                    committer.shoot_store(
                        &mut **ownership,
                        &mut **sync_view,
                        *release_view,
                    );
                }
            ),
        );
    }

    const _: u8 = AVX2;
    const _: u8 = SSE42;
    const _: u8 = NOP;
}
