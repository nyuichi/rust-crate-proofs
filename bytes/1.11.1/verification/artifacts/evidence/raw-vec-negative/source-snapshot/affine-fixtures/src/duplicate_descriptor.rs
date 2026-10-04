use crate::raw_vec::RawAllocation;

#[allow(dead_code)]
pub(crate) fn duplicate_descriptor(raw: RawAllocation) {
    let first_owner = raw;
    let second_owner = raw;
    let _ = (first_owner, second_owner);
}
