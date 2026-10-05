use creusot_std::prelude::*;

// STD-CONVERT-01: external standard-library contract, not a bytes contract.
// The fixed Rust source uses an always-Ok conversion on 64-bit targets,
// and an upper-bound check on 32/16-bit targets. The contract covers both.
extern_spec! {
    impl core::convert::TryFrom<u64> for usize {
        #[ensures(match result {
            Ok(v) => v@ == value@ && value@ <= usize::MAX@,
            Err(_) => value@ > usize::MAX@
        })]
        fn try_from(value: u64) -> Result<usize, <usize as core::convert::TryFrom<u64>>::Error>;
    }
}

