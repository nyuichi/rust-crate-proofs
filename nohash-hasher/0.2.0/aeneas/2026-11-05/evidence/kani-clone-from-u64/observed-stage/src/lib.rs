#[cfg(all(kani, feature="expect-no-debug", debug_assertions))]
compile_error!("Expected debug_assertions=false");
#[cfg(all(kani, not(feature="expect-no-debug"), not(debug_assertions)))]
compile_error!("Expected debug_assertions=true");
#[cfg(kani)]
mod checks {
use core::hash::Hasher;
use nohash_hasher::NoHashHasher;
fn reachable(value:u64,written:bool)->NoHashHasher<u64> {
 let mut h=NoHashHasher::default();if written {h.write_u64(value);}h
}
#[kani::proof]
fn actual_std_clone_from() {
 let src_value:u64=kani::any();let src_written:bool=kani::any();
 let dst_value:u64=kani::any();let dst_written:bool=kani::any();
 let src=reachable(src_value,src_written);let mut dst=reachable(dst_value,dst_written);
 let expected_src=if src_written {src_value}else{0};
 assert_eq!(src.finish(),expected_src);
 assert_eq!(dst.finish(),if dst_written {dst_value}else{0});
 <NoHashHasher<u64> as Clone>::clone_from(&mut dst,&src);
 assert_eq!(dst.finish(),expected_src);assert_eq!(src.finish(),expected_src);
 let expected_flag=if cfg!(debug_assertions) {Some(src_written)} else {None};
 assert_eq!(nohash_hasher::__verification_observe_u64(&dst),(expected_src,expected_flag));
 assert_eq!(nohash_hasher::__verification_observe_u64(&src),(expected_src,expected_flag));
}
}
