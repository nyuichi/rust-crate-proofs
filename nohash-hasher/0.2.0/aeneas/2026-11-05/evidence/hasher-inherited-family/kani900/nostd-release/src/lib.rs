#![feature(hasher_prefixfree_extras)]
const _: () = assert!(!cfg!(debug_assertions));
use nohash_hasher::NoHashHasher;
use core::hash::Hasher;

#[cfg(kani)]
#[kani::proof]
fn all_enabled_initial_prefix() {
 let length:usize=kani::any();
 let prior:u64=kani::any();
 let written:bool=kani::any();
 { let mut h=NoHashHasher::<u8>::default(); if written { h.write_u64(prior); } Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<u16>::default(); if written { h.write_u64(prior); } Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<u32>::default(); if written { h.write_u64(prior); } Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<u64>::default(); if written { h.write_u64(prior); } Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<usize>::default(); if written { h.write_u64(prior); } Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<i8>::default(); if written { h.write_u64(prior); } Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<i16>::default(); if written { h.write_u64(prior); } Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<i32>::default(); if written { h.write_u64(prior); } Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<i64>::default(); if written { h.write_u64(prior); } Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
 { let mut h=NoHashHasher::<isize>::default(); if written { h.write_u64(prior); } Hasher::write_length_prefix(&mut h,length); assert_eq!(h.finish(),length as u64); }
}
