#![feature(hasher_prefixfree_extras)]
use nohash_hasher::NoHashHasher;
use core::hash::Hasher;

#[cfg(kani)]
#[kani::proof]
fn all_enabled_initial_prefix() {
 let length:usize=kani::any();
 { let mut h=NoHashHasher::<u8>::default(); Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<u16>::default(); Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<u32>::default(); Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<u64>::default(); Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<usize>::default(); Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<i8>::default(); Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<i16>::default(); Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<i32>::default(); Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<i64>::default(); Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<isize>::default(); Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
}
