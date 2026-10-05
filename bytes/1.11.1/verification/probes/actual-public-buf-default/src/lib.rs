//! Proof probe for the actual public `Buf::try_get_u8` default and the minimum
//! public trait laws needed to establish it.
#![allow(unexpected_cfgs, dead_code)]

#[path = "../../../../src/slice_ops.rs"]
mod slice_ops;

include!(concat!(env!("OUT_DIR"), "/actual_public_buf.rs"));
