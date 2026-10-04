use creusot_std::prelude::Ghost;

use crate::raw_vec::{PhysicalRegion, Recovery};

#[allow(dead_code)]
pub(crate) fn reuse_after_split(capabilities: Ghost<(Recovery, PhysicalRegion)>) {
    let (recovery, region) = capabilities.split();
    let duplicate = capabilities;
    let _ = (recovery, region, duplicate);
}
