use crate::prelude::*;
#[cfg(creusot)]
use core::borrow::Borrow;

/// The logical relation between an owned value's model and its borrowed
/// value's model. This records the content relationship only; it does not
/// establish Rust's stronger Eq, Ord, and Hash coherence requirements for a
/// `Borrow` implementation.
#[cfg_attr(not(creusot), allow(dead_code))]
pub trait BorrowModel<Rhs> {
    #[logic]
    fn borrowed_model(self, rhs: Rhs) -> bool;
}

/// Same-type borrows preserve the model by ordinary logical equality.
impl<T> BorrowModel<T> for T {
    #[logic(open)]
    fn borrowed_model(self, rhs: T) -> bool {
        pearlite! { self == rhs }
    }
}

// "In particular Eq, Ord and Hash must be equivalent for borrowed and owned values:
// x.borrow() == y.borrow() should give the same result as x == y."
// https://doc.rust-lang.org/std/borrow/trait.Borrow.html

extern_spec! {
    mod core {
        mod borrow {
            trait Borrow<Borrowed>
            where Borrowed: ?Sized
            {
                #[ensures(self.deep_model().borrowed_model(result.deep_model()))]
                fn borrow(&self) -> &Borrowed
                where
                    Self: DeepModel,
                    Borrowed: DeepModel,
                    Self::DeepModelTy: BorrowModel<Borrowed::DeepModelTy>;
            }
        }
    }
}
