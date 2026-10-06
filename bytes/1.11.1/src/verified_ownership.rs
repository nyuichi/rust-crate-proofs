//! Canonical physical dependencies for the modified implementation.
//! These paths share bodies and contracts with the existing ownership bridges.
#[path = "ownership_proof/bound_ptr.rs"]
pub(crate) mod bound_ptr;
#[path = "ownership_proof/owned_region.rs"]
pub(crate) mod owned_region;
#[path = "ownership_proof/raw_vec.rs"]
pub(crate) mod raw_vec;
#[path = "ownership_proof/frozen_region.rs"]
pub(crate) mod frozen_region;
