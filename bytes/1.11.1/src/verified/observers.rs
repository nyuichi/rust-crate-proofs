//! Equality observers backed by the verified standard slice comparison contract.

use creusot_std::prelude::*;
use core::cmp::Ordering;

use super::exclusive::ExclusiveBytes;

impl DeepModel for ExclusiveBytes {
    type DeepModelTy = Seq<Int>;

    #[logic(open)]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { self@.map(|byte: u8| byte@) }
    }
}

impl PartialEq for ExclusiveBytes {
    #[ensures(result == (self.deep_model() == other.deep_model()))]
    fn eq(&self, other: &Self) -> bool {
        <Self as AsRef<[u8]>>::as_ref(self) == <Self as AsRef<[u8]>>::as_ref(other)
    }
}

impl Eq for ExclusiveBytes {}

impl PartialOrd for ExclusiveBytes {
    #[ensures(result == (*self).deep_model().partial_cmp_log((*other).deep_model()))]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(<Self as Ord>::cmp(self, other))
    }
}

impl Ord for ExclusiveBytes {
    #[ensures(result == (*self).deep_model().cmp_log((*other).deep_model()))]
    fn cmp(&self, other: &Self) -> Ordering {
        <Self as AsRef<[u8]>>::as_ref(self).cmp(<Self as AsRef<[u8]>>::as_ref(other))
    }
}

/// Compares two exclusive owners, then releases both allocations explicitly.
#[ensures(result == (left.deep_model() == right.deep_model()))]
pub fn equal_then_close(left: ExclusiveBytes, right: ExclusiveBytes) -> bool {
    let equal = left == right;
    left.close();
    right.close();
    equal
}

/// Orders two exclusive owners, then explicitly releases both allocations.
#[ensures(result == left.deep_model().cmp_log(right.deep_model()))]
pub fn cmp_then_close(left: ExclusiveBytes, right: ExclusiveBytes) -> Ordering {
    let ordering = left.cmp(&right);
    left.close();
    right.close();
    ordering
}

#[cfg(all(test, not(creusot), feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn ordering_matches_lexicographic_slice_order_and_closes() {
        let left = ExclusiveBytes::copy_from_slice(b"byte");
        let right = ExclusiveBytes::copy_from_slice(b"bytes");
        assert_eq!(cmp_then_close(left, right), Ordering::Less);
    }
}
