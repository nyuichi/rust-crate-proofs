use creusot_std::prelude::{ensures, requires};
use creusot_std::std::mem::size_of_logic;

/// Consumer probe for the allocation-bound helper used by actual
/// `HeaderMap::len`.
#[requires(size_of_logic::<T>() > 0)]
#[ensures(result@ <= isize::MAX@)]
pub fn generic_vec_len<T>(values: &Vec<T>) -> usize {
    crate::header::map::allocation_len_bound(values);
    values.len()
}

/// Two non-ZST Vec lengths each fit in `isize::MAX`; their sum therefore fits
/// in `usize::MAX` on Rust's paired pointer-sized integer types.
#[requires(size_of_logic::<T>() > 0)]
#[ensures(result@ == left@.len() + right@.len())]
pub fn sum_nonzero_vec_lengths<T>(left: &Vec<T>, right: &Vec<T>) -> usize {
    crate::header::map::allocation_len_bound(left);
    crate::header::map::allocation_len_bound(right);
    left.len() + right.len()
}

/// Concrete non-ZST instantiation of the permission-derived length bound.
#[ensures(result@ <= isize::MAX@)]
pub fn u8_vec_len(values: &Vec<u8>) -> usize {
    crate::header::map::allocation_len_bound(values);
    values.len()
}

/// ZST vectors have no allocation-byte bound; retain only their sequence-view
/// equality and avoid imposing the non-ZST `isize::MAX` condition.
#[ensures(result@ == values@.len())]
pub fn unit_vec_len(values: &Vec<()>) -> usize {
    values.len()
}
