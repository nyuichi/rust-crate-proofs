use crate::prelude::*;
#[cfg(creusot)]
use core::fmt::{Debug, Display, Result};

impl View for core::fmt::Formatter<'_> {
    type ViewTy = Seq<u8>;

    /// Characters successfully written to this formatter so far.
    #[trusted]
    #[logic(opaque)]
    fn view(self) -> Self::ViewTy {
        dead
    }
}

impl DeepModel for core::fmt::Formatter<'_> {
    type DeepModelTy = Seq<Int>;

    #[logic]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { self@.map(|byte: u8| byte@) }
    }
}

/// Formatting may append output, but it never changes the bytes already emitted.
#[logic(open)]
pub fn formatter_extends(before: Seq<Int>, after: Seq<Int>) -> bool {
    pearlite! { exists<suffix: Seq<Int>> after == before.concat(suffix) }
}

extern_spec! {
    mod core {
        mod fmt {
            trait Display {
                #[ensures(formatter_extends(formatter.deep_model(), (^formatter).deep_model()))]
                fn fmt(
                    &self,
                    formatter: &mut core::fmt::Formatter<'_>,
                ) -> core::fmt::Result;
            }

            trait Debug {
                #[ensures(formatter_extends(formatter.deep_model(), (^formatter).deep_model()))]
                fn fmt(
                    &self,
                    formatter: &mut core::fmt::Formatter<'_>,
                ) -> core::fmt::Result;
            }
        }
    }
}

// `Display for str` delegates to `Formatter::pad`. The pinned implementation
// writes slices of the input and requested fill padding through the formatter buffer;
// failures may leave partial output but never alter its existing prefix.
extern_spec! {
    impl<'a> core::fmt::Formatter<'a> {
        #[ensures(formatter_extends(self.deep_model(), (^self).deep_model()))]
        fn pad(&mut self, data: &str) -> Result;
    }
}

extern_spec! {
    impl core::fmt::Display for str {
        #[ensures(formatter_extends(formatter.deep_model(), (^formatter).deep_model()))]
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            formatter.pad(self)
        }
    }
}

// These contracts cover the complete builder borrow, including the nested
// `Debug::fmt` callback used by `field`; each call may append output but keeps
// the original formatter as a prefix. This models append preservation, not
// the exact formatted text, and only covers the builder methods listed here.
extern_spec! {
    impl<'a> core::fmt::Formatter<'a> {
        #[ensures(formatter_extends(self.deep_model(), (^self).deep_model()))]
        fn debug_struct<'b>(
            &'b mut self,
            name: &str,
        ) -> core::fmt::DebugStruct<'b, 'a>;
    }

    impl<'a, 'b> core::fmt::DebugStruct<'a, 'b> {
        #[ensures(inv(^self))]
        fn field<'c>(
            &'c mut self,
            name: &str,
            value: &dyn Debug,
        ) -> &'c mut Self;

        #[ensures(inv(^self))]
        fn finish(&mut self) -> Result;
    }

    impl<'a> core::fmt::Formatter<'a> {
        #[ensures(formatter_extends(self.deep_model(), (^self).deep_model()))]
        fn debug_tuple<'b>(
            &'b mut self,
            name: &str,
        ) -> core::fmt::DebugTuple<'b, 'a>;
    }

    impl<'a, 'b> core::fmt::DebugTuple<'a, 'b> {
        #[ensures(inv(^self))]
        fn field<'c>(
            &'c mut self,
            value: &dyn Debug,
        ) -> &'c mut Self;

        #[ensures(inv(^self))]
        fn finish(&mut self) -> Result;
    }
}

extern_spec! {
    impl<'a> core::fmt::Formatter<'a> {
        #[ensures(exists<i> 0 <= i && i <= data@.to_bytes().len()
            && (^self).deep_model() == self.deep_model().concat(
                data@.to_bytes().subsequence(0, i).map(|byte: u8| byte@))
            && match result { Ok(_) => i == data@.to_bytes().len(), Err(_) => true })]
        fn write_str(&mut self, data: &str) -> Result;

        #[ensures(formatter_extends(self.deep_model(), (^self).deep_model()))]
        fn debug_struct_field1_finish<'b>(
            &'b mut self,
            name: &str,
            name1: &str,
            value1: &dyn Debug,
        ) -> Result;

        #[ensures(formatter_extends(self.deep_model(), (^self).deep_model()))]
        fn debug_struct_field2_finish<'b>(
            &'b mut self,
            name: &str,
            name1: &str,
            value1: &dyn Debug,
            name2: &str,
            value2: &dyn Debug,
        ) -> Result;

        #[ensures(formatter_extends(self.deep_model(), (^self).deep_model()))]
        fn debug_struct_field3_finish<'b>(
            &'b mut self,
            name: &str,
            name1: &str,
            value1: &dyn Debug,
            name2: &str,
            value2: &dyn Debug,
            name3: &str,
            value3: &dyn Debug,
        ) -> Result;

        #[ensures(formatter_extends(self.deep_model(), (^self).deep_model()))]
        fn debug_struct_field4_finish<'b>(
            &'b mut self,
            name: &str,
            name1: &str,
            value1: &dyn Debug,
            name2: &str,
            value2: &dyn Debug,
            name3: &str,
            value3: &dyn Debug,
            name4: &str,
            value4: &dyn Debug,
        ) -> Result;

        #[ensures(formatter_extends(self.deep_model(), (^self).deep_model()))]
        fn debug_struct_field5_finish<'b>(
            &'b mut self,
            name: &str,
            name1: &str,
            value1: &dyn Debug,
            name2: &str,
            value2: &dyn Debug,
            name3: &str,
            value3: &dyn Debug,
            name4: &str,
            value4: &dyn Debug,
            name5: &str,
            value5: &dyn Debug,
        ) -> Result;

        #[ensures(formatter_extends(self.deep_model(), (^self).deep_model()))]
        fn debug_struct_fields_finish<'b>(
            &'b mut self,
            name: &str,
            names: &[&str],
            values: &[&dyn Debug],
        ) -> Result;

        #[ensures(formatter_extends(self.deep_model(), (^self).deep_model()))]
        fn debug_tuple_field1_finish<'b>(
            &'b mut self,
            name: &str,
            value1: &dyn Debug,
        ) -> Result;

        #[ensures(formatter_extends(self.deep_model(), (^self).deep_model()))]
        fn debug_tuple_field2_finish<'b>(
            &'b mut self,
            name: &str,
            value1: &dyn Debug,
            value2: &dyn Debug,
        ) -> Result;

        #[ensures(formatter_extends(self.deep_model(), (^self).deep_model()))]
        fn debug_tuple_field3_finish<'b>(
            &'b mut self,
            name: &str,
            value1: &dyn Debug,
            value2: &dyn Debug,
            value3: &dyn Debug,
        ) -> Result;

        #[ensures(formatter_extends(self.deep_model(), (^self).deep_model()))]
        fn debug_tuple_field4_finish<'b>(
            &'b mut self,
            name: &str,
            value1: &dyn Debug,
            value2: &dyn Debug,
            value3: &dyn Debug,
            value4: &dyn Debug,
        ) -> Result;

        #[ensures(formatter_extends(self.deep_model(), (^self).deep_model()))]
        fn debug_tuple_field5_finish<'b>(
            &'b mut self,
            name: &str,
            value1: &dyn Debug,
            value2: &dyn Debug,
            value3: &dyn Debug,
            value4: &dyn Debug,
            value5: &dyn Debug,
        ) -> Result;

        #[ensures(formatter_extends(self.deep_model(), (^self).deep_model()))]
        fn debug_tuple_fields_finish<'b>(
            &'b mut self,
            name: &str,
            values: &[&dyn Debug],
        ) -> Result;
    }
}

extern_spec! {
    impl<'a> core::fmt::Arguments<'a> {
        #[check(ghost)]
        fn from_str(s: &'static str) -> Self;
    }
}
