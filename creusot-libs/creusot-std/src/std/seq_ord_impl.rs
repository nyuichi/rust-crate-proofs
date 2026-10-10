//! `OrdLogic` implementation backed by the independently proved comparator.

use crate::{logic::OrdLogic, prelude::*, std::seq_ord::*};
use core::cmp::Ordering;

impl<T: OrdLogic> OrdLogic for Seq<T> {
    #[logic(open)]
    fn cmp_log(self, other: Self) -> Ordering {
        seq_cmp(self, other)
    }

    #[logic(law)]
    #[ensures(x.le_log(y) == (x.cmp_log(y) != Ordering::Greater))]
    fn cmp_le_log(x: Self, y: Self) {}

    #[logic(law)]
    #[ensures(x.lt_log(y) == (x.cmp_log(y) == Ordering::Less))]
    fn cmp_lt_log(x: Self, y: Self) {}

    #[logic(law)]
    #[ensures(x.ge_log(y) == (x.cmp_log(y) != Ordering::Less))]
    fn cmp_ge_log(x: Self, y: Self) {}

    #[logic(law)]
    #[ensures(x.gt_log(y) == (x.cmp_log(y) == Ordering::Greater))]
    fn cmp_gt_log(x: Self, y: Self) {}

    #[logic(law)]
    #[ensures(x.cmp_log(x) == Ordering::Equal)]
    fn refl(x: Self) {
        seq_eq_cmp(x, x);
    }

    #[logic(law)]
    #[requires(x.cmp_log(y) == order)]
    #[requires(y.cmp_log(z) == order)]
    #[ensures(x.cmp_log(z) == order)]
    fn trans(x: Self, y: Self, z: Self, order: Ordering) {
        seq_cmp_trans(x, y, z, order);
    }

    #[logic(law)]
    #[requires(x.cmp_log(y) == Ordering::Less)]
    #[ensures(y.cmp_log(x) == Ordering::Greater)]
    fn antisym1(x: Self, y: Self) {
        seq_reverse_cmp(x, y);
    }

    #[logic(law)]
    #[requires(x.cmp_log(y) == Ordering::Greater)]
    #[ensures(y.cmp_log(x) == Ordering::Less)]
    fn antisym2(x: Self, y: Self) {
        seq_reverse_cmp(x, y);
    }

    #[logic(law)]
    #[ensures((x == y) == (x.cmp_log(y) == Ordering::Equal))]
    fn eq_cmp(x: Self, y: Self) {
        seq_eq_cmp(x, y);
    }
}
