use nohash_hasher::{IntMap,IntSet,BuildNoHashHasher};
// Positive: valid alias constructors preserve the fixed nohash builder.
pub fn supported(){
 let _:IntMap<u64,u64>=IntMap::default();
 let _:IntMap<u64,u64>=IntMap::with_hasher(BuildNoHashHasher::default());
 let _:IntSet<u64>=IntSet::default();
 let _:IntSet<u64>=IntSet::with_hasher(BuildNoHashHasher::default());
}
#[cfg(feature="negative")]
mod rejected {
 use super::*;
 fn map_new()->IntMap<u64,u64>{IntMap::<u64,u64>::new()}
 fn map_capacity()->IntMap<u64,u64>{IntMap::<u64,u64>::with_capacity(1)}
 fn map_from()->IntMap<u64,u64>{<IntMap<u64,u64> as From<[(u64,u64);1]>>::from([(0,0)])}
 fn set_new()->IntSet<u64>{IntSet::<u64>::new()}
 fn set_capacity()->IntSet<u64>{IntSet::<u64>::with_capacity(1)}
 fn set_from()->IntSet<u64>{<IntSet<u64> as From<[u64;1]>>::from([0])}
}
