#![feature(hasher_prefixfree_extras)]
const _: ()=assert!(!cfg!(debug_assertions));
pub fn prefix<T:nohash_hasher::IsEnabled>(hasher:&mut nohash_hasher::NoHashHasher<T>,length:usize) {
 core::hash::Hasher::write_length_prefix(hasher,length)
}
pub fn wide_unsigned<T:nohash_hasher::IsEnabled>(hasher:&mut nohash_hasher::NoHashHasher<T>,value:u128) {
 core::hash::Hasher::write_u128(hasher,value)
}
pub fn wide_signed<T:nohash_hasher::IsEnabled>(hasher:&mut nohash_hasher::NoHashHasher<T>,value:i128) {
 core::hash::Hasher::write_i128(hasher,value)
}
