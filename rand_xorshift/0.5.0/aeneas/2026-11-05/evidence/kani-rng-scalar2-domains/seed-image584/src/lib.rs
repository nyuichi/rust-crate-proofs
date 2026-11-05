#[cfg(kani)] mod checks {
use rand_xorshift::XorShiftRng; use rand_core::{Rng,TryRng,SeedableRng};
#[kani::proof] fn original_rng_next32_delegates_tryrng(){let seed:[u8;16]=kani::any();let mut a=XorShiftRng::from_seed(seed);let mut b=a.clone();let x=<XorShiftRng as Rng>::next_u32(&mut a);let y=match <XorShiftRng as TryRng>::try_next_u32(&mut b){Ok(x)=>x,Err(e)=>match e{}};assert_eq!(x,y);assert!(a==b);}
#[kani::proof] fn original_rng_next64_delegates_tryrng(){let seed:[u8;16]=kani::any();let mut a=XorShiftRng::from_seed(seed);let mut b=a.clone();let x=<XorShiftRng as Rng>::next_u64(&mut a);let y=match <XorShiftRng as TryRng>::try_next_u64(&mut b){Ok(x)=>x,Err(e)=>match e{}};assert_eq!(x,y);assert!(a==b);}
}
