#[cfg(kani)]
mod range_checks {
    #[kani::proof]
    fn original_rand32_float_range_only() {
        let mut rng=oorandom::Rand32::from_state((kani::any(),kani::any()));
        let actual=rng.rand_float();
        assert!(actual>=0.0 && actual<1.0);
    }
    #[kani::proof]
    fn original_rand64_float_range_only() {
        let mut rng=oorandom::Rand64::from_state((kani::any(),kani::any()));
        let actual=rng.rand_float();
        assert!(actual>=0.0 && actual<1.0);
    }
}
