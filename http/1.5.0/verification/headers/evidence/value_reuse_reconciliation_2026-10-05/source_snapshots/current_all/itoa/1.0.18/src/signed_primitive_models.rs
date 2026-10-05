use creusot_std::prelude::{check, ensures, extern_spec};
use creusot_std::prelude::View;

// Narrow model of core's primitive `unsigned_abs`. In `int_macros.rs`, this
// is implemented as `self.wrapping_abs() as UnsignedT`, including MIN.
extern_spec! {
    impl i8 {
        #[check(terminates)]
        #[ensures(result@ == if self@ < 0 { -self@ } else { self@ })]
        fn unsigned_abs(self) -> u8;
    }
    impl i16 {
        #[check(terminates)]
        #[ensures(result@ == if self@ < 0 { -self@ } else { self@ })]
        fn unsigned_abs(self) -> u16;
    }
    impl i32 {
        #[check(terminates)]
        #[ensures(result@ == if self@ < 0 { -self@ } else { self@ })]
        fn unsigned_abs(self) -> u32;
    }
    impl i64 {
        #[check(terminates)]
        #[ensures(result@ == if self@ < 0 { -self@ } else { self@ })]
        fn unsigned_abs(self) -> u64;
    }
    impl i128 {
        #[check(terminates)]
        #[ensures(result@ == if self@ < 0 { -self@ } else { self@ })]
        fn unsigned_abs(self) -> u128;
    }
}
