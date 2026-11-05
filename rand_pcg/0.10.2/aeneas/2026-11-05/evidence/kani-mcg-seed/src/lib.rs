#[cfg(kani)]
mod checks {
use rand_pcg::rand_core::SeedableRng;
#[kani::proof]
fn original_mcg128xsl64_seed_state(){
let bytes:[u8;16]=kani::any();
let rng=rand_pcg::Mcg128Xsl64::from_seed(bytes);
let state=(u128::from(bytes[0])<<0) | (u128::from(bytes[1])<<8) | (u128::from(bytes[2])<<16) | (u128::from(bytes[3])<<24) | (u128::from(bytes[4])<<32) | (u128::from(bytes[5])<<40) | (u128::from(bytes[6])<<48) | (u128::from(bytes[7])<<56) | (u128::from(bytes[8])<<64) | (u128::from(bytes[9])<<72) | (u128::from(bytes[10])<<80) | (u128::from(bytes[11])<<88) | (u128::from(bytes[12])<<96) | (u128::from(bytes[13])<<104) | (u128::from(bytes[14])<<112) | (u128::from(bytes[15])<<120);
assert_eq!(rng.state(),state|1);
}
}
