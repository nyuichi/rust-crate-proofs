#![allow(unexpected_cfgs)]

#[cfg(target_arch = "x86_64")]
mod sse_prefix {
    use creusot_std::logic::ops::NthBitLogic;
    use creusot_std::prelude::{
        bitwise_proof, ensures, extern_spec, logic, requires, DeepModel, Int,
    };
    use creusot_std::std::num::NumExt;
    use core::arch::x86_64::{
        __m128i, _mm_andnot_si128, _mm_cmpeq_epi8, _mm_max_epu8, _mm_movemask_epi8,
        _mm_set1_epi8,
    };

    /// Logical signed-byte lanes for the opaque hardware vector type.
    #[logic(opaque)]
    pub fn lanes(v: __m128i) -> [i8; 16] {
        dead
    }

    #[logic(open)]
    pub fn byte_value(lane: i8) -> Int {
        if lane.deep_model() < 0 {
            lane.deep_model() + 256
        } else {
            lane.deep_model()
        }
    }

    #[logic(open)]
    pub fn uri_lane_allowed(lane: i8) -> bool {
        let b = byte_value(lane);
        b >= 0x21 && b != 0x7f
    }

    #[logic(open)]
    pub fn sign_bit(lane: i8) -> Int {
        if lane.deep_model() < 0 { 1 } else { 0 }
    }

    // Scalar, body-proved facts used to keep the vector mask proof out of the
    // arithmetic details of signed-byte interpretation and instruction rows.
    #[ensures(0 <= byte_value(lane) && byte_value(lane) <= 255)]
    #[ensures((byte_value(lane) == 33) == (lane == 33i8))]
    #[ensures((byte_value(lane) == 127) == (lane == 127i8))]
    fn signed_byte_value_facts(lane: i8) {}

    #[ensures(
        ((if byte_value(lane) >= 33 { lane } else { 33i8 }) == lane)
            == (byte_value(lane) >= 33)
    )]
    fn unsigned_max_lane_threshold(lane: i8) {}

    #[requires((a == -1i8 || a == 0i8) && (b == -1i8 || b == 0i8))]
    #[ensures((sign_bit(!a & b) == 1) == (a == 0i8 && b == -1i8))]
    #[bitwise_proof]
    fn andnot_signbit_truth_table(a: i8, b: i8) {}

    #[requires(0 <= value@ && value@ <= 65535)]
    #[ensures((value as u16)@ == value@)]
    #[ensures(forall<i: Int> 0 <= i && i < 16 ==>
        (value as u16).nth_bit(i) == value.nth_bit(i))]
    #[bitwise_proof]
    fn movemask_narrow_preserves_bits(value: i32) {}

    #[ensures(forall<i: Int> 0 <= i && i < 16 ==>
        (!value).nth_bit(i) == !value.nth_bit(i))]
    #[bitwise_proof]
    fn mask_complement_preserves_low_bits(value: u16) {}

    // Smoke targets for the local Why3-driver overlay. The first three
    // functions state true bit-index facts; the `expected_invalid_*` targets
    // intentionally state false facts and should produce counterexamples when
    // checked with the repaired driver. Translation/dumps alone do not check
    // either expectation.
    #[ensures((1u16).nth_bit(0))]
    #[ensures(!(1u16).nth_bit(15))]
    #[ensures((0x8000u16).nth_bit(15))]
    #[ensures(!(0x8000u16).nth_bit(0))]
    #[ensures((!1u16).nth_bit(15))]
    #[ensures(!(!1u16).nth_bit(0))]
    #[bitwise_proof]
    fn u16_nth_constant_bit_smoke() {}

    #[ensures(!(1u16).nth_bit(-1))]
    #[ensures(!(1u16).nth_bit(16))]
    #[ensures(!(1u16).nth_bit(65536))]
    #[ensures(!(0xffffu16).nth_bit(-1))]
    #[bitwise_proof]
    fn u16_nth_out_of_bounds_smoke() {}

    #[ensures((1u32).nth_bit(0))]
    #[ensures(!(1u32).nth_bit(31))]
    #[ensures((0x8000_0000u32).nth_bit(31))]
    #[ensures(!(0x8000_0000u32).nth_bit(0))]
    #[ensures(!(1u32).nth_bit(-1))]
    #[ensures(!(1u32).nth_bit(32))]
    #[ensures(!(1u32).nth_bit(4294967296))]
    #[bitwise_proof]
    fn u32_nth_boundary_smoke() {}

    #[ensures((1u16).nth_bit(15))]
    #[bitwise_proof]
    fn expected_invalid_u16_wrong_bit_index() {}

    #[ensures((0x8000u16).nth_bit(0))]
    #[bitwise_proof]
    fn expected_invalid_u16_wrong_high_bit() {}

    #[ensures((1u16).nth_bit(-1))]
    #[bitwise_proof]
    fn expected_invalid_u16_negative_index_wrap() {}

    #[ensures((1u16).nth_bit(16))]
    #[bitwise_proof]
    fn expected_invalid_u16_width_index_wrap() {}

    #[ensures((1u16).nth_bit(65536))]
    #[bitwise_proof]
    fn expected_invalid_u16_two_to_width_wrap() {}

    #[ensures((0x10000i32 as u16).nth_bit(0))]
    #[bitwise_proof]
    fn expected_invalid_u16_cast_overflow_nonzero() {}

    #[ensures((-1i32 as u16)@ == (-1i32)@)]
    #[bitwise_proof]
    fn expected_invalid_u16_cast_numeric_identity() {}

    #[ensures((!1u16).nth_bit(0))]
    #[bitwise_proof]
    fn expected_invalid_u16_complement_bit_zero() {}

    #[ensures((1u32).nth_bit(31))]
    #[bitwise_proof]
    fn expected_invalid_u32_wrong_bit_index() {}

    #[ensures((0x8000_0000u32).nth_bit(0))]
    #[bitwise_proof]
    fn expected_invalid_u32_wrong_high_bit() {}

    #[ensures((1u32).nth_bit(-1))]
    #[bitwise_proof]
    fn expected_invalid_u32_negative_index_wrap() {}

    #[ensures((1u32).nth_bit(32))]
    #[bitwise_proof]
    fn expected_invalid_u32_width_index_wrap() {}

    #[ensures((1u32).nth_bit(4294967296))]
    #[bitwise_proof]
    fn expected_invalid_u32_two_to_width_wrap() {}


    extern_spec! {
        mod core {
            mod arch {
                mod x86_64 {
                    #[ensures(lanes(result)@.len() == 16)]
                    #[ensures(forall<i: Int> 0 <= i && i < 16 ==> lanes(result)@[i] == a)]
                    unsafe fn _mm_set1_epi8(a: i8) -> core::arch::x86_64::__m128i;

                    #[ensures(lanes(result)@.len() == 16)]
                    #[ensures(forall<i: Int> 0 <= i && i < 16 ==>
                        lanes(result)@[i] ==
                            if byte_value(lanes(a)@[i]) >= byte_value(lanes(b)@[i])
                            { lanes(a)@[i] } else { lanes(b)@[i] })]
                    unsafe fn _mm_max_epu8(
                        a: core::arch::x86_64::__m128i,
                        b: core::arch::x86_64::__m128i,
                    ) -> core::arch::x86_64::__m128i;

                    #[ensures(lanes(result)@.len() == 16)]
                    #[ensures(forall<i: Int> 0 <= i && i < 16 ==>
                        lanes(result)@[i] == if lanes(a)@[i] == lanes(b)@[i] { -1i8 } else { 0i8 })]
                    unsafe fn _mm_cmpeq_epi8(
                        a: core::arch::x86_64::__m128i,
                        b: core::arch::x86_64::__m128i,
                    ) -> core::arch::x86_64::__m128i;

                    #[ensures(lanes(result)@.len() == 16)]
                    #[ensures(forall<i: Int> 0 <= i && i < 16 ==>
                        lanes(result)@[i] == (!lanes(a)@[i] & lanes(b)@[i]))]
                    #[bitwise_proof]
                    unsafe fn _mm_andnot_si128(
                        a: core::arch::x86_64::__m128i,
                        b: core::arch::x86_64::__m128i,
                    ) -> core::arch::x86_64::__m128i;

                    #[ensures(0 <= result@ && result@ <= 65535)]
                    #[ensures(forall<i: Int> 0 <= i && i < 16 ==>
                        result.nth_bit(i) == (sign_bit(lanes(v)@[i]) == 1))]
                    unsafe fn _mm_movemask_epi8(v: core::arch::x86_64::__m128i) -> i32;
                }
            }
        }
    }

    // Bridge the std model's exact dynamic-shift characterization into direct
    // bit-index facts for the mask prefix contract. This body is checked.
    #[requires(count == value.trailing_zeros_logic())]
    #[ensures(count@ <= 16)]
    #[ensures(forall<i: Int> 0 <= i && i < count@ ==> !value.nth_bit(i))]
    #[ensures(count@ == 16 || value.nth_bit(count@))]
    #[bitwise_proof]
    fn trailing_zeros_shift_to_nth(value: u16, count: u32) {}

    /// The exact mask-building expression sequence from httparse's SSE URI
    /// block predicate, with only the unaligned load factored out as `dat`.
    #[target_feature(enable = "sse4.2")]
    #[ensures(0 <= result@ && result@ <= 65535)]
    #[ensures(forall<i: Int> 0 <= i && i < 16 ==>
        result.nth_bit(i) == uri_lane_allowed(lanes(dat)@[i]))]
    #[bitwise_proof]
    unsafe fn uri_allowed_mask_16_sse(dat: __m128i) -> u16 {
        let del: __m128i = _mm_set1_epi8(0x7f);
        let low: __m128i = _mm_set1_epi8(0x21);
        let lo = _mm_cmpeq_epi8(_mm_max_epu8(dat, low), dat);
        let is_del = _mm_cmpeq_epi8(dat, del);
        let bit = _mm_andnot_si128(is_del, lo);
        let raw_mask = _mm_movemask_epi8(bit);
        movemask_narrow_preserves_bits(raw_mask);
        raw_mask as u16
    }

    /// Convert the run of low one bits into the URI prefix length using the
    /// standard method and a checked shift-to-bit-index bridge.
    #[ensures(result@ <= 16)]
    #[ensures(forall<i: Int> 0 <= i && i < result@ ==> mask.nth_bit(i))]
    #[ensures(result@ < 16 ==> !mask.nth_bit(result@))]
    #[bitwise_proof]
    fn prefix_len_from_mask(mask: u16) -> usize {
        let inverted = !mask;
        mask_complement_preserves_low_bits(mask);
        let advance = inverted.trailing_zeros();
        trailing_zeros_shift_to_nth(inverted, advance);
        advance as usize
    }

    /// Composition boundary for the two independent proof components.
    #[target_feature(enable = "sse4.2")]
    #[ensures(result@ <= 16)]
    #[ensures(forall<i: Int> 0 <= i && i < result@ ==> uri_lane_allowed(lanes(dat)@[i]))]
    #[ensures(result@ < 16 ==> !uri_lane_allowed(lanes(dat)@[result@]))]
    pub unsafe fn match_uri_char_16_sse_pure(dat: __m128i) -> usize {
        let mask = uri_allowed_mask_16_sse(dat);
        prefix_len_from_mask(mask)
    }
}
