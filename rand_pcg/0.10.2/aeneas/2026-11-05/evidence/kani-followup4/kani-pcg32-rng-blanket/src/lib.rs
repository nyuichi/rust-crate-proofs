#[cfg(kani)]mod checks{
use rand_pcg::Lcg64Xsh32;use rand_core::{Rng,TryRng};
#[kani::proof]fn original_rng_next32_delegates_tryrng(){let mut a=Lcg64Xsh32::from_state(kani::any(),kani::any());let mut b=a.clone();let x=<Lcg64Xsh32 as Rng>::next_u32(&mut a);let y=match<Lcg64Xsh32 as TryRng>::try_next_u32(&mut b){Ok(x)=>x,Err(e)=>match e{}};assert_eq!(x,y);assert!(a==b);}
#[kani::proof]fn original_rng_next64_delegates_tryrng(){let mut a=Lcg64Xsh32::from_state(kani::any(),kani::any());let mut b=a.clone();let x=<Lcg64Xsh32 as Rng>::next_u64(&mut a);let y=match<Lcg64Xsh32 as TryRng>::try_next_u64(&mut b){Ok(x)=>x,Err(e)=>match e{}};assert_eq!(x,y);assert!(a==b);}
}
