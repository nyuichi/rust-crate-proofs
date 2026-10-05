use core::ops::{Index, IndexMut, RangeFrom};
use creusot_std::prelude::{check, ensures, extern_spec, logic, requires, View};
use creusot_std::std::slice::SliceIndexSpec;

trait FormatterIndex {
    #[logic]
    fn formatter_index_allowed(self) -> bool;
}
impl FormatterIndex for usize {
    #[logic(open)]
    fn formatter_index_allowed(self) -> bool { true }
}
impl FormatterIndex for RangeFrom<usize> {
    #[logic(open)]
    fn formatter_index_allowed(self) -> bool { true }
}

// Narrow contracts for the core array indexing and slice-to-array operations
// used by the original signed writer. The index whitelist prevents this model
// from applying to unrelated array index types.
extern_spec! {
    impl<T: View, I: SliceIndexSpec<[T]> + FormatterIndex, const N: usize> IndexMut<I>
        for [T; N]
    {
        #[check(terminates)]
        #[requires(index.formatter_index_allowed())]
        #[requires(index.in_bounds(self@))]
        #[ensures(index.has_value(self@, *result))]
        #[ensures(index.has_value((&^self)@, ^result))]
        #[ensures(index.resolve_elswhere(self@, (&^self)@))]
        #[ensures((&^self)@.len() == self@.len())]
        fn index_mut(&mut self, index: I) -> &mut <[T; N] as Index<I>>::Output;
    }

    impl<'a, T, const N: usize> TryFrom<&'a mut [T]> for &'a mut [T; N] {
        #[check(terminates)]
        #[requires(slice@.len() == N@)]
        #[ensures(exists<array: &mut [T; N]>
            result == Ok(array) && array@ == slice@ && (^array)@ == (^slice)@ &&
            forall<i: creusot_std::prelude::Int> 0 <= i && i < N@ ==>
                array@[i] == slice@[i] && (^array)@[i] == (^slice)@[i])]
        fn try_from(
            slice: &'a mut [T],
        ) -> Result<&'a mut [T; N], core::array::TryFromSliceError>;
    }
}
