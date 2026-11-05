// Rust normalization helpers copy only the original float normalization,
// not the RNG implementation. Source-closure comparison is recorded separately.
pub fn normalize32(mut word:u32)->f32 {const TOTAL_BITS:u32=32;const PRECISION:u32=core::f32::MANTISSA_DIGITS;const MANTISSA_SCALE:f32=1.0/((1u32<<PRECISION)as f32);word>>=TOTAL_BITS-PRECISION;word as f32*MANTISSA_SCALE}
pub fn normalize64(mut word:u64)->f64 {const TOTAL_BITS:u32=64;const PRECISION:u32=core::f64::MANTISSA_DIGITS;const MANTISSA_SCALE:f64=1.0/((1u64<<PRECISION)as f64);word>>=TOTAL_BITS-PRECISION;word as f64*MANTISSA_SCALE}
pub fn bits32(n:u32)->u32 {if n==0{0}else{let k=31-n.leading_zeros();let exponent=k+127-24;(exponent<<23)|((n<<(23-k))&0x7fffff)}}
pub fn bits64(n:u64)->u64 {if n==0{0}else{let k=63-n.leading_zeros();let exponent=k+1023-53;((exponent as u64)<<52)|((n<<(52-k))&0x000fffffffffffff)}}
#[cfg(kani)] mod checks {
 use super::*;
 #[kani::proof] fn original32_consumes_one_word_rust_formula(){let state:(u64,u64)=(kani::any(),kani::any());let mut a=oorandom::Rand32::from_state(state);let mut b=a.clone();let f=a.rand_float();let w=b.rand_u32();assert_eq!(f.to_bits(),normalize32(w).to_bits());assert_eq!(a.state(),b.state());}
 #[kani::proof] fn original64_consumes_one_word_rust_formula(){let state:(u128,u128)=(kani::any(),kani::any());let mut a=oorandom::Rand64::from_state(state);let mut b=a.clone();let f=a.rand_float();let w=b.rand_u64();assert_eq!(f.to_bits(),normalize64(w).to_bits());assert_eq!(a.state(),b.state());}
 #[kani::proof] fn pure32_normalization_independent_ieee(){let w:u32=kani::any();let f=normalize32(w);assert_eq!(f.to_bits(),bits32(w>>8));assert!(f>=0.0&&f<1.0);}
 #[kani::proof] fn pure64_normalization_independent_ieee(){let w:u64=kani::any();let f=normalize64(w);assert_eq!(f.to_bits(),bits64(w>>11));assert!(f>=0.0&&f<1.0);}
}
