#![allow(unexpected_cfgs, dead_code)]

#[cfg(feature = "missing_deep_model")]
mod missing;
#[cfg(all(not(feature = "missing_deep_model"), feature = "omit_rhs_model_bound"))]
mod modeled_gap;
#[cfg(all(not(feature = "missing_deep_model"), not(feature = "omit_rhs_model_bound")))]
mod modeled;
