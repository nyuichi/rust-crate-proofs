use vstd::prelude::*;

verus! {
use vstd::string::StringSliceAdditionalSpecFns;
use vstd::utf8::{is_leading_byte_width_1, partial_valid_utf8, partial_valid_utf8_extend_ascii_block, valid_utf8};

fn ascii_bytes_to_str<'a>(bytes: &'a [u8]) -> (result: &'a str)
    requires
        forall|i: int| 0 <= i < bytes@.len() ==> bytes@[i] <= 127,
    ensures
        result.spec_bytes() =~= bytes@,
{
    proof {
        assert forall|i: int| 0 <= i < bytes@.len() implies is_leading_byte_width_1(bytes@[i]) by {
            assert(0 <= bytes@[i]);
        }

        assert(partial_valid_utf8(bytes@, 0)) by {
            assert(bytes@[..0] =~= seq![]);
            assert(valid_utf8(seq![]));
        }

        partial_valid_utf8_extend_ascii_block(bytes@, 0, bytes@.len() as int);
        assert(bytes@[..bytes@.len()] =~= bytes@);
        assert(valid_utf8(bytes@));
    }

    unsafe { str::from_utf8_unchecked(bytes) }
}
}
