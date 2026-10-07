//! Modified bytes API: alloc core with optional scoped std sharing.
#![allow(missing_debug_implementations, unexpected_cfgs, dead_code)]
pub mod exclusive;
pub mod cursor;
pub mod float_bits;
pub mod adapters;
pub mod encoding_spec;
pub mod iteration;
pub mod conversions;
pub mod observers;
mod writes;
#[cfg(feature = "std")]
mod write_readback;
#[cfg(feature = "std")]
pub use write_readback::scoped_write_read_u16_be;
#[cfg(feature = "std")]
mod scoped;
#[cfg(feature = "std")]
pub use scoped::*;
