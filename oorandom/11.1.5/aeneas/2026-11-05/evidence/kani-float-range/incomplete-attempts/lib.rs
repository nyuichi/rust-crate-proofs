#[cfg(kani)]
mod checks {
    #[kani::proof]
    fn original_rand32_float_bits_range_and_state() {
        let state:u64=kani::any();
        let increment:u64=kani::any();
        let mut rng=oorandom::Rand32::from_state((state,increment));
        let word=((((state>>18)^state)>>27) as u32).rotate_right((state>>59) as u32);
        let expected=((word>>8) as f32)*f32::from_bits((127u32-24)<<23);
        let actual=rng.rand_float();
        assert_eq!(actual.to_bits(),expected.to_bits());
        assert!(actual>=0.0 && actual<1.0);
        assert_eq!(rng.state(),(state.wrapping_mul(6364136223846793005).wrapping_add(increment),increment));
    }
    #[kani::proof]
    fn original_rand64_float_bits_range_and_state() {
        let state:u128=kani::any();
        let increment:u128=kani::any();
        let mut rng=oorandom::Rand64::from_state((state,increment));
        let word=((((state>>29)^state)>>58) as u64).rotate_right((state>>122) as u32);
        let expected=((word>>11) as f64)*f64::from_bits((1023u64-53)<<52);
        let actual=rng.rand_float();
        assert_eq!(actual.to_bits(),expected.to_bits());
        assert!(actual>=0.0 && actual<1.0);
        assert_eq!(rng.state(),(state.wrapping_mul(47026247687942121848144207491837523525).wrapping_add(increment),increment));
    }
}
#[cfg(kani)]
mod composition_checks {
    #[kani::proof]
    fn original_rand32_float_composes_original_word() {
        let state:u64=kani::any();let increment:u64=kani::any();
        let mut actual_rng=oorandom::Rand32::from_state((state,increment));
        let mut word_rng=oorandom::Rand32::from_state((state,increment));
        let word=word_rng.rand_u32();
        let expected=((word>>8) as f32)*f32::from_bits((127u32-24)<<23);
        let actual=actual_rng.rand_float();
        assert_eq!(actual.to_bits(),expected.to_bits());
        assert!(actual>=0.0 && actual<1.0);
        assert_eq!(actual_rng.state(),word_rng.state());
    }
    #[kani::proof]
    fn original_rand64_float_composes_original_word() {
        let state:u128=kani::any();let increment:u128=kani::any();
        let mut actual_rng=oorandom::Rand64::from_state((state,increment));
        let mut word_rng=oorandom::Rand64::from_state((state,increment));
        let word=word_rng.rand_u64();
        let expected=((word>>11) as f64)*f64::from_bits((1023u64-53)<<52);
        let actual=actual_rng.rand_float();
        assert_eq!(actual.to_bits(),expected.to_bits());
        assert!(actual>=0.0 && actual<1.0);
        assert_eq!(actual_rng.state(),word_rng.state());
    }
}
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
