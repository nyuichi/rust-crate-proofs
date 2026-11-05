#[cfg(kani)]
mod checks {
 // Independent IEEE754 encoding of exact integer n * 2^-precision.
 // All nonzero values are normal, with no rounding or subnormal case.
 fn bits32(n:u32)->u32 {if n==0 {0}else{let k=31-n.leading_zeros();let exponent=k+127-24; (exponent<<23)|((n<<(23-k))&0x7fffff)}}
 fn bits64(n:u64)->u64 {if n==0 {0}else{let k=63-n.leading_zeros();let exponent=k+1023-53; (exponent<<52)|((n<<(52-k))&0x000fffffffffffff)}}
 #[kani::proof]
 fn original_rand32_float_public_word_exact_bits(){
  let state:(u64,u64)=(kani::any(),kani::any());let mut a=oorandom::Rand32::from_state(state);let mut b=a.clone();
  let actual=a.rand_float();let word=b.rand_u32();let integer=word>>8;
  assert_eq!(actual.to_bits(),bits32(integer));assert!(actual>=0.0&&actual<1.0);assert_eq!(a.state(),b.state());
 }
 #[kani::proof]
 fn original_rand64_float_public_word_exact_bits(){
  let state:(u128,u128)=(kani::any(),kani::any());let mut a=oorandom::Rand64::from_state(state);let mut b=a.clone();
  let actual=a.rand_float();let word=b.rand_u64();let integer=word>>11;
  assert_eq!(actual.to_bits(),bits64(integer));assert!(actual>=0.0&&actual<1.0);assert_eq!(a.state(),b.state());
 }
}
