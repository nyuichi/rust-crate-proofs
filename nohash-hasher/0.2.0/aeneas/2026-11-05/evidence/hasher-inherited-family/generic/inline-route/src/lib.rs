#![feature(hasher_prefixfree_extras)]
pub fn prefix<T:nohash_hasher::IsEnabled>(hasher:&mut nohash_hasher::NoHashHasher<T>,length:usize) {
 core::hash::Hasher::write_length_prefix(hasher,length)
}
