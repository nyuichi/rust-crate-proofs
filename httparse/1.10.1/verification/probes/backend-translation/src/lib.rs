#![allow(unexpected_cfgs)]

#[cfg(feature = "swar")]
mod swar {
    use creusot_std::prelude::ensures;

    const BLOCK_SIZE: usize = core::mem::size_of::<usize>();
    type ByteBlock = [u8; BLOCK_SIZE];

    const fn uniform_block(b: u8) -> usize {
        (b as u64 * 0x01_01_01_01_01_01_01_01) as usize
    }

    // Isolates the exact word-level mask expressions after byte-to-word loading.
    // This does not include the runtime representation bridge from [u8; 8].
    #[ensures(result@ >= 0)]
    pub fn match_uri_mask_word(x: usize) -> usize {
        const BM: usize = uniform_block(0x21);
        const ONE: usize = uniform_block(0x01);
        const DEL: usize = uniform_block(0x7f);
        const M128: usize = uniform_block(128);

        let lt = x.wrapping_sub(BM) & !x;
        let xor_del = x ^ DEL;
        let eq_del = xor_del.wrapping_sub(ONE) & !xor_del;
        (lt | eq_del) & M128
    }

    // One-block body copied structurally from httparse 1.10.1's SWAR URI scanner.
    #[ensures(result@ >= 0)]
    pub fn match_uri_char_8_swar(block: ByteBlock) -> usize {
        const BM: usize = uniform_block(0x21);
        const ONE: usize = uniform_block(0x01);
        const DEL: usize = uniform_block(0x7f);
        const M128: usize = uniform_block(128);

        let x = usize::from_ne_bytes(block);
        let lt = x.wrapping_sub(BM) & !x;
        let xor_del = x ^ DEL;
        let eq_del = xor_del.wrapping_sub(ONE) & !xor_del;
        offsetnz((lt | eq_del) & M128)
    }

    #[ensures(result@ >= 0)]
    fn offsetnz(block: usize) -> usize {
        if block == 0 {
            return BLOCK_SIZE;
        }
        for (i, b) in block.to_ne_bytes().iter().copied().enumerate() {
            if b != 0 {
                return i;
            }
        }
        BLOCK_SIZE
    }
}

#[cfg(all(feature = "sse", any(target_arch = "x86", target_arch = "x86_64")))]
mod sse {
    use creusot_std::prelude::ensures;
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    // Isolates one actual intrinsic call used by httparse's SSE mask scanner.
    #[target_feature(enable = "sse4.2")]
    #[ensures(result@ >= 0)]
    pub unsafe fn sse_intrinsic_probe(v: __m128i) -> i32 {
        _mm_movemask_epi8(v)
    }
}

#[cfg(all(feature = "dispatch", any(target_arch = "x86", target_arch = "x86_64")))]
mod dispatch {
    use creusot_std::prelude::ensures;
    use std::sync::atomic::{AtomicU8, Ordering};

    const SSE42: u8 = 2;
    const NOP: u8 = 3;
    static RUNTIME_FEATURE: AtomicU8 = AtomicU8::new(0);

    fn detect_runtime_feature() -> u8 {
        if is_x86_feature_detected!("sse4.2") {
            SSE42
        } else {
            NOP
        }
    }

    // Mirrors httparse's relaxed load / detect / store shape. Concurrent first
    // callers may race to store the same runtime-detected value.
    #[ensures(result@ == 2 || result@ == 3)]
    pub fn get_runtime_feature() -> u8 {
        let mut feature = RUNTIME_FEATURE.load(Ordering::Relaxed);
        if feature == 0 {
            feature = detect_runtime_feature();
            RUNTIME_FEATURE.store(feature, Ordering::Relaxed);
        }
        feature
    }
}

#[cfg(all(feature = "sse-exact", target_arch = "x86_64"))]
mod sse_exact {
    use creusot_std::prelude::{ensures, extern_spec, logic, DeepModel, Int};
    use core::arch::x86_64::{__m128i, _mm_movemask_epi8, _mm_set1_epi8};

    /// Abstract unsigned byte-lane view of the hardware vector. Exact
    /// intrinsic contracts below define the relation used by this local TCB.
    #[logic(opaque)]
    fn lanes(v: __m128i) -> [i8; 16] {
        dead
    }

    #[logic(open)]
    pub fn sign_bit(lane: i8) -> Int {
        let lane_value = lane.deep_model();
        if lane_value < 0 { 1 } else { 0 }
    }

    #[logic(open)]
    fn movemask_model(lanes: [i8; 16]) -> Int {
        sign_bit(lanes[0])
            + 2 * sign_bit(lanes[1])
            + 4 * sign_bit(lanes[2])
            + 8 * sign_bit(lanes[3])
            + 16 * sign_bit(lanes[4])
            + 32 * sign_bit(lanes[5])
            + 64 * sign_bit(lanes[6])
            + 128 * sign_bit(lanes[7])
            + 256 * sign_bit(lanes[8])
            + 512 * sign_bit(lanes[9])
            + 1024 * sign_bit(lanes[10])
            + 2048 * sign_bit(lanes[11])
            + 4096 * sign_bit(lanes[12])
            + 8192 * sign_bit(lanes[13])
            + 16384 * sign_bit(lanes[14])
            + 32768 * sign_bit(lanes[15])
    }

    extern_spec! {
        mod core {
            mod arch {
                mod x86_64 {
                    #[ensures(lanes(result)@.len() == 16)]
                    #[ensures(forall<i: Int> 0 <= i && i < 16 ==> lanes(result)@[i] == a)]
                    unsafe fn _mm_set1_epi8(a: i8) -> core::arch::x86_64::__m128i;

                    #[ensures(result@ == movemask_model(lanes(v)))]
                    unsafe fn _mm_movemask_epi8(v: core::arch::x86_64::__m128i) -> i32;
                }
            }
        }
    }

    #[target_feature(enable = "sse4.2")]
    #[ensures(result@ == 65535)]
    pub unsafe fn all_ones_movemask() -> i32 {
        let value = _mm_set1_epi8(-1);
        _mm_movemask_epi8(value)
    }
}
