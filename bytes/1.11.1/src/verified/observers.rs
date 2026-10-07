//! Equality observers backed by the verified standard slice comparison contract.

use creusot_std::prelude::*;

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

/// Compares two exclusive owners, then releases both allocations explicitly.
#[ensures(result == (left.deep_model() == right.deep_model()))]
pub fn equal_then_close(left: ExclusiveBytes, right: ExclusiveBytes) -> bool {
    let equal = left == right;
    left.close();
    right.close();
    equal
}
