use rand_core::{Rng,TryRng};
use core::convert::Infallible;
pub fn original_fill_root<R:TryRng<Error=Infallible>+?Sized>(rng:&mut R,buf:&mut[u8]){<R as Rng>::fill_bytes(rng,buf)}
