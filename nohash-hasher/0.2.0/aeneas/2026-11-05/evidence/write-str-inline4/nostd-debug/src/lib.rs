#![feature(hasher_prefixfree_extras)]
const _: ()=assert!(cfg!(debug_assertions));
pub fn write_text<T:nohash_hasher::IsEnabled>(hasher:&mut nohash_hasher::NoHashHasher<T>,text:&str) {
 core::hash::Hasher::write_str(hasher,text)
}
