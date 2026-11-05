use rand_core::SeedableRng;
#[kani::proof]
fn original_splitmix64_from_seed_words(){
 let bytes:[u8;8]=kani::any();
 let rng=<crate::SplitMix64 as SeedableRng>::from_seed(bytes);
 let words=crate::splitmix64::proof_seed_words(&rng);
 let effective=bytes;
 assert_eq!(words[0],((u64::from(effective[0])) << 0) | ((u64::from(effective[1])) << 8) | ((u64::from(effective[2])) << 16) | ((u64::from(effective[3])) << 24) | ((u64::from(effective[4])) << 32) | ((u64::from(effective[5])) << 40) | ((u64::from(effective[6])) << 48) | ((u64::from(effective[7])) << 56));
}
#[kani::proof]
fn original_xoroshiro128plus_from_seed_words(){
 let bytes:[u8;16]=kani::any();
 let rng=<crate::Xoroshiro128Plus as SeedableRng>::from_seed(bytes);
 let words=crate::xoroshiro128plus::proof_seed_words(&rng);
 let effective=if bytes==[0;16]{[175, 205, 29, 123, 57, 168, 32, 226, 244, 101, 185, 161, 106, 158, 120, 110]}else{bytes};
 assert_eq!(words[0],((u64::from(effective[0])) << 0) | ((u64::from(effective[1])) << 8) | ((u64::from(effective[2])) << 16) | ((u64::from(effective[3])) << 24) | ((u64::from(effective[4])) << 32) | ((u64::from(effective[5])) << 40) | ((u64::from(effective[6])) << 48) | ((u64::from(effective[7])) << 56));
 assert_eq!(words[1],((u64::from(effective[8])) << 0) | ((u64::from(effective[9])) << 8) | ((u64::from(effective[10])) << 16) | ((u64::from(effective[11])) << 24) | ((u64::from(effective[12])) << 32) | ((u64::from(effective[13])) << 40) | ((u64::from(effective[14])) << 48) | ((u64::from(effective[15])) << 56));
}
#[kani::proof]
fn original_xoroshiro128plusplus_from_seed_words(){
 let bytes:[u8;16]=kani::any();
 let rng=<crate::Xoroshiro128PlusPlus as SeedableRng>::from_seed(bytes);
 let words=crate::xoroshiro128plusplus::proof_seed_words(&rng);
 let effective=if bytes==[0;16]{[175, 205, 29, 123, 57, 168, 32, 226, 244, 101, 185, 161, 106, 158, 120, 110]}else{bytes};
 assert_eq!(words[0],((u64::from(effective[0])) << 0) | ((u64::from(effective[1])) << 8) | ((u64::from(effective[2])) << 16) | ((u64::from(effective[3])) << 24) | ((u64::from(effective[4])) << 32) | ((u64::from(effective[5])) << 40) | ((u64::from(effective[6])) << 48) | ((u64::from(effective[7])) << 56));
 assert_eq!(words[1],((u64::from(effective[8])) << 0) | ((u64::from(effective[9])) << 8) | ((u64::from(effective[10])) << 16) | ((u64::from(effective[11])) << 24) | ((u64::from(effective[12])) << 32) | ((u64::from(effective[13])) << 40) | ((u64::from(effective[14])) << 48) | ((u64::from(effective[15])) << 56));
}
#[kani::proof]
fn original_xoroshiro128starstar_from_seed_words(){
 let bytes:[u8;16]=kani::any();
 let rng=<crate::Xoroshiro128StarStar as SeedableRng>::from_seed(bytes);
 let words=crate::xoroshiro128starstar::proof_seed_words(&rng);
 let effective=if bytes==[0;16]{[175, 205, 29, 123, 57, 168, 32, 226, 244, 101, 185, 161, 106, 158, 120, 110]}else{bytes};
 assert_eq!(words[0],((u64::from(effective[0])) << 0) | ((u64::from(effective[1])) << 8) | ((u64::from(effective[2])) << 16) | ((u64::from(effective[3])) << 24) | ((u64::from(effective[4])) << 32) | ((u64::from(effective[5])) << 40) | ((u64::from(effective[6])) << 48) | ((u64::from(effective[7])) << 56));
 assert_eq!(words[1],((u64::from(effective[8])) << 0) | ((u64::from(effective[9])) << 8) | ((u64::from(effective[10])) << 16) | ((u64::from(effective[11])) << 24) | ((u64::from(effective[12])) << 32) | ((u64::from(effective[13])) << 40) | ((u64::from(effective[14])) << 48) | ((u64::from(effective[15])) << 56));
}
#[kani::proof]
fn original_xoroshiro64star_from_seed_words(){
 let bytes:[u8;8]=kani::any();
 let rng=<crate::Xoroshiro64Star as SeedableRng>::from_seed(bytes);
 let words=crate::xoroshiro64star::proof_seed_words(&rng);
 let effective=if bytes==[0;8]{[175, 205, 29, 123, 57, 168, 32, 226]}else{bytes};
 assert_eq!(words[0],((u32::from(effective[0])) << 0) | ((u32::from(effective[1])) << 8) | ((u32::from(effective[2])) << 16) | ((u32::from(effective[3])) << 24));
 assert_eq!(words[1],((u32::from(effective[4])) << 0) | ((u32::from(effective[5])) << 8) | ((u32::from(effective[6])) << 16) | ((u32::from(effective[7])) << 24));
}
#[kani::proof]
fn original_xoroshiro64starstar_from_seed_words(){
 let bytes:[u8;8]=kani::any();
 let rng=<crate::Xoroshiro64StarStar as SeedableRng>::from_seed(bytes);
 let words=crate::xoroshiro64starstar::proof_seed_words(&rng);
 let effective=if bytes==[0;8]{[175, 205, 29, 123, 57, 168, 32, 226]}else{bytes};
 assert_eq!(words[0],((u32::from(effective[0])) << 0) | ((u32::from(effective[1])) << 8) | ((u32::from(effective[2])) << 16) | ((u32::from(effective[3])) << 24));
 assert_eq!(words[1],((u32::from(effective[4])) << 0) | ((u32::from(effective[5])) << 8) | ((u32::from(effective[6])) << 16) | ((u32::from(effective[7])) << 24));
}
#[kani::proof]
fn original_xoshiro128plus_from_seed_words(){
 let bytes:[u8;16]=kani::any();
 let rng=<crate::Xoshiro128Plus as SeedableRng>::from_seed(bytes);
 let words=crate::xoshiro128plus::proof_seed_words(&rng);
 let effective=if bytes==[0;16]{[175, 205, 29, 123, 57, 168, 32, 226, 244, 101, 185, 161, 106, 158, 120, 110]}else{bytes};
 assert_eq!(words[0],((u32::from(effective[0])) << 0) | ((u32::from(effective[1])) << 8) | ((u32::from(effective[2])) << 16) | ((u32::from(effective[3])) << 24));
 assert_eq!(words[1],((u32::from(effective[4])) << 0) | ((u32::from(effective[5])) << 8) | ((u32::from(effective[6])) << 16) | ((u32::from(effective[7])) << 24));
 assert_eq!(words[2],((u32::from(effective[8])) << 0) | ((u32::from(effective[9])) << 8) | ((u32::from(effective[10])) << 16) | ((u32::from(effective[11])) << 24));
 assert_eq!(words[3],((u32::from(effective[12])) << 0) | ((u32::from(effective[13])) << 8) | ((u32::from(effective[14])) << 16) | ((u32::from(effective[15])) << 24));
}
#[kani::proof]
fn original_xoshiro128plusplus_from_seed_words(){
 let bytes:[u8;16]=kani::any();
 let rng=<crate::Xoshiro128PlusPlus as SeedableRng>::from_seed(bytes);
 let words=crate::xoshiro128plusplus::proof_seed_words(&rng);
 let effective=if bytes==[0;16]{[175, 205, 29, 123, 57, 168, 32, 226, 244, 101, 185, 161, 106, 158, 120, 110]}else{bytes};
 assert_eq!(words[0],((u32::from(effective[0])) << 0) | ((u32::from(effective[1])) << 8) | ((u32::from(effective[2])) << 16) | ((u32::from(effective[3])) << 24));
 assert_eq!(words[1],((u32::from(effective[4])) << 0) | ((u32::from(effective[5])) << 8) | ((u32::from(effective[6])) << 16) | ((u32::from(effective[7])) << 24));
 assert_eq!(words[2],((u32::from(effective[8])) << 0) | ((u32::from(effective[9])) << 8) | ((u32::from(effective[10])) << 16) | ((u32::from(effective[11])) << 24));
 assert_eq!(words[3],((u32::from(effective[12])) << 0) | ((u32::from(effective[13])) << 8) | ((u32::from(effective[14])) << 16) | ((u32::from(effective[15])) << 24));
}
#[kani::proof]
fn original_xoshiro128starstar_from_seed_words(){
 let bytes:[u8;16]=kani::any();
 let rng=<crate::Xoshiro128StarStar as SeedableRng>::from_seed(bytes);
 let words=crate::xoshiro128starstar::proof_seed_words(&rng);
 let effective=if bytes==[0;16]{[175, 205, 29, 123, 57, 168, 32, 226, 244, 101, 185, 161, 106, 158, 120, 110]}else{bytes};
 assert_eq!(words[0],((u32::from(effective[0])) << 0) | ((u32::from(effective[1])) << 8) | ((u32::from(effective[2])) << 16) | ((u32::from(effective[3])) << 24));
 assert_eq!(words[1],((u32::from(effective[4])) << 0) | ((u32::from(effective[5])) << 8) | ((u32::from(effective[6])) << 16) | ((u32::from(effective[7])) << 24));
 assert_eq!(words[2],((u32::from(effective[8])) << 0) | ((u32::from(effective[9])) << 8) | ((u32::from(effective[10])) << 16) | ((u32::from(effective[11])) << 24));
 assert_eq!(words[3],((u32::from(effective[12])) << 0) | ((u32::from(effective[13])) << 8) | ((u32::from(effective[14])) << 16) | ((u32::from(effective[15])) << 24));
}
#[kani::proof]
fn original_xoshiro256plus_from_seed_words(){
 let bytes:[u8;32]=kani::any();
 let rng=<crate::Xoshiro256Plus as SeedableRng>::from_seed(bytes);
 let words=crate::xoshiro256plus::proof_seed_words(&rng);
 let effective=if bytes==[0;32]{[175, 205, 29, 123, 57, 168, 32, 226, 244, 101, 185, 161, 106, 158, 120, 110, 79, 69, 9, 128, 24, 93, 196, 6, 236, 129, 76, 114, 168, 184, 139, 248]}else{bytes};
 assert_eq!(words[0],((u64::from(effective[0])) << 0) | ((u64::from(effective[1])) << 8) | ((u64::from(effective[2])) << 16) | ((u64::from(effective[3])) << 24) | ((u64::from(effective[4])) << 32) | ((u64::from(effective[5])) << 40) | ((u64::from(effective[6])) << 48) | ((u64::from(effective[7])) << 56));
 assert_eq!(words[1],((u64::from(effective[8])) << 0) | ((u64::from(effective[9])) << 8) | ((u64::from(effective[10])) << 16) | ((u64::from(effective[11])) << 24) | ((u64::from(effective[12])) << 32) | ((u64::from(effective[13])) << 40) | ((u64::from(effective[14])) << 48) | ((u64::from(effective[15])) << 56));
 assert_eq!(words[2],((u64::from(effective[16])) << 0) | ((u64::from(effective[17])) << 8) | ((u64::from(effective[18])) << 16) | ((u64::from(effective[19])) << 24) | ((u64::from(effective[20])) << 32) | ((u64::from(effective[21])) << 40) | ((u64::from(effective[22])) << 48) | ((u64::from(effective[23])) << 56));
 assert_eq!(words[3],((u64::from(effective[24])) << 0) | ((u64::from(effective[25])) << 8) | ((u64::from(effective[26])) << 16) | ((u64::from(effective[27])) << 24) | ((u64::from(effective[28])) << 32) | ((u64::from(effective[29])) << 40) | ((u64::from(effective[30])) << 48) | ((u64::from(effective[31])) << 56));
}
#[kani::proof]
fn original_xoshiro256plusplus_from_seed_words(){
 let bytes:[u8;32]=kani::any();
 let rng=<crate::Xoshiro256PlusPlus as SeedableRng>::from_seed(bytes);
 let words=crate::xoshiro256plusplus::proof_seed_words(&rng);
 let effective=if bytes==[0;32]{[175, 205, 29, 123, 57, 168, 32, 226, 244, 101, 185, 161, 106, 158, 120, 110, 79, 69, 9, 128, 24, 93, 196, 6, 236, 129, 76, 114, 168, 184, 139, 248]}else{bytes};
 assert_eq!(words[0],((u64::from(effective[0])) << 0) | ((u64::from(effective[1])) << 8) | ((u64::from(effective[2])) << 16) | ((u64::from(effective[3])) << 24) | ((u64::from(effective[4])) << 32) | ((u64::from(effective[5])) << 40) | ((u64::from(effective[6])) << 48) | ((u64::from(effective[7])) << 56));
 assert_eq!(words[1],((u64::from(effective[8])) << 0) | ((u64::from(effective[9])) << 8) | ((u64::from(effective[10])) << 16) | ((u64::from(effective[11])) << 24) | ((u64::from(effective[12])) << 32) | ((u64::from(effective[13])) << 40) | ((u64::from(effective[14])) << 48) | ((u64::from(effective[15])) << 56));
 assert_eq!(words[2],((u64::from(effective[16])) << 0) | ((u64::from(effective[17])) << 8) | ((u64::from(effective[18])) << 16) | ((u64::from(effective[19])) << 24) | ((u64::from(effective[20])) << 32) | ((u64::from(effective[21])) << 40) | ((u64::from(effective[22])) << 48) | ((u64::from(effective[23])) << 56));
 assert_eq!(words[3],((u64::from(effective[24])) << 0) | ((u64::from(effective[25])) << 8) | ((u64::from(effective[26])) << 16) | ((u64::from(effective[27])) << 24) | ((u64::from(effective[28])) << 32) | ((u64::from(effective[29])) << 40) | ((u64::from(effective[30])) << 48) | ((u64::from(effective[31])) << 56));
}
#[kani::proof]
fn original_xoshiro256starstar_from_seed_words(){
 let bytes:[u8;32]=kani::any();
 let rng=<crate::Xoshiro256StarStar as SeedableRng>::from_seed(bytes);
 let words=crate::xoshiro256starstar::proof_seed_words(&rng);
 let effective=if bytes==[0;32]{[175, 205, 29, 123, 57, 168, 32, 226, 244, 101, 185, 161, 106, 158, 120, 110, 79, 69, 9, 128, 24, 93, 196, 6, 236, 129, 76, 114, 168, 184, 139, 248]}else{bytes};
 assert_eq!(words[0],((u64::from(effective[0])) << 0) | ((u64::from(effective[1])) << 8) | ((u64::from(effective[2])) << 16) | ((u64::from(effective[3])) << 24) | ((u64::from(effective[4])) << 32) | ((u64::from(effective[5])) << 40) | ((u64::from(effective[6])) << 48) | ((u64::from(effective[7])) << 56));
 assert_eq!(words[1],((u64::from(effective[8])) << 0) | ((u64::from(effective[9])) << 8) | ((u64::from(effective[10])) << 16) | ((u64::from(effective[11])) << 24) | ((u64::from(effective[12])) << 32) | ((u64::from(effective[13])) << 40) | ((u64::from(effective[14])) << 48) | ((u64::from(effective[15])) << 56));
 assert_eq!(words[2],((u64::from(effective[16])) << 0) | ((u64::from(effective[17])) << 8) | ((u64::from(effective[18])) << 16) | ((u64::from(effective[19])) << 24) | ((u64::from(effective[20])) << 32) | ((u64::from(effective[21])) << 40) | ((u64::from(effective[22])) << 48) | ((u64::from(effective[23])) << 56));
 assert_eq!(words[3],((u64::from(effective[24])) << 0) | ((u64::from(effective[25])) << 8) | ((u64::from(effective[26])) << 16) | ((u64::from(effective[27])) << 24) | ((u64::from(effective[28])) << 32) | ((u64::from(effective[29])) << 40) | ((u64::from(effective[30])) << 48) | ((u64::from(effective[31])) << 56));
}
#[kani::proof]
fn original_xoshiro512plus_from_seed_words(){
 let bytes:[u8;64]=kani::any();
 let rng=<crate::Xoshiro512Plus as SeedableRng>::from_seed(crate::Seed512(bytes));
 let words=crate::xoshiro512plus::proof_seed_words(&rng);
 let effective=if bytes==[0;64]{[175, 205, 29, 123, 57, 168, 32, 226, 244, 101, 185, 161, 106, 158, 120, 110, 79, 69, 9, 128, 24, 93, 196, 6, 236, 129, 76, 114, 168, 184, 139, 248, 155, 116, 168, 81, 106, 137, 57, 27, 234, 162, 126, 116, 12, 159, 203, 83, 225, 50, 69, 31, 190, 154, 130, 44, 60, 171, 22, 201, 58, 19, 132, 197]}else{bytes};
 assert_eq!(words[0],((u64::from(effective[0])) << 0) | ((u64::from(effective[1])) << 8) | ((u64::from(effective[2])) << 16) | ((u64::from(effective[3])) << 24) | ((u64::from(effective[4])) << 32) | ((u64::from(effective[5])) << 40) | ((u64::from(effective[6])) << 48) | ((u64::from(effective[7])) << 56));
 assert_eq!(words[1],((u64::from(effective[8])) << 0) | ((u64::from(effective[9])) << 8) | ((u64::from(effective[10])) << 16) | ((u64::from(effective[11])) << 24) | ((u64::from(effective[12])) << 32) | ((u64::from(effective[13])) << 40) | ((u64::from(effective[14])) << 48) | ((u64::from(effective[15])) << 56));
 assert_eq!(words[2],((u64::from(effective[16])) << 0) | ((u64::from(effective[17])) << 8) | ((u64::from(effective[18])) << 16) | ((u64::from(effective[19])) << 24) | ((u64::from(effective[20])) << 32) | ((u64::from(effective[21])) << 40) | ((u64::from(effective[22])) << 48) | ((u64::from(effective[23])) << 56));
 assert_eq!(words[3],((u64::from(effective[24])) << 0) | ((u64::from(effective[25])) << 8) | ((u64::from(effective[26])) << 16) | ((u64::from(effective[27])) << 24) | ((u64::from(effective[28])) << 32) | ((u64::from(effective[29])) << 40) | ((u64::from(effective[30])) << 48) | ((u64::from(effective[31])) << 56));
 assert_eq!(words[4],((u64::from(effective[32])) << 0) | ((u64::from(effective[33])) << 8) | ((u64::from(effective[34])) << 16) | ((u64::from(effective[35])) << 24) | ((u64::from(effective[36])) << 32) | ((u64::from(effective[37])) << 40) | ((u64::from(effective[38])) << 48) | ((u64::from(effective[39])) << 56));
 assert_eq!(words[5],((u64::from(effective[40])) << 0) | ((u64::from(effective[41])) << 8) | ((u64::from(effective[42])) << 16) | ((u64::from(effective[43])) << 24) | ((u64::from(effective[44])) << 32) | ((u64::from(effective[45])) << 40) | ((u64::from(effective[46])) << 48) | ((u64::from(effective[47])) << 56));
 assert_eq!(words[6],((u64::from(effective[48])) << 0) | ((u64::from(effective[49])) << 8) | ((u64::from(effective[50])) << 16) | ((u64::from(effective[51])) << 24) | ((u64::from(effective[52])) << 32) | ((u64::from(effective[53])) << 40) | ((u64::from(effective[54])) << 48) | ((u64::from(effective[55])) << 56));
 assert_eq!(words[7],((u64::from(effective[56])) << 0) | ((u64::from(effective[57])) << 8) | ((u64::from(effective[58])) << 16) | ((u64::from(effective[59])) << 24) | ((u64::from(effective[60])) << 32) | ((u64::from(effective[61])) << 40) | ((u64::from(effective[62])) << 48) | ((u64::from(effective[63])) << 56));
}
#[kani::proof]
fn original_xoshiro512plusplus_from_seed_words(){
 let bytes:[u8;64]=kani::any();
 let rng=<crate::Xoshiro512PlusPlus as SeedableRng>::from_seed(crate::Seed512(bytes));
 let words=crate::xoshiro512plusplus::proof_seed_words(&rng);
 let effective=if bytes==[0;64]{[175, 205, 29, 123, 57, 168, 32, 226, 244, 101, 185, 161, 106, 158, 120, 110, 79, 69, 9, 128, 24, 93, 196, 6, 236, 129, 76, 114, 168, 184, 139, 248, 155, 116, 168, 81, 106, 137, 57, 27, 234, 162, 126, 116, 12, 159, 203, 83, 225, 50, 69, 31, 190, 154, 130, 44, 60, 171, 22, 201, 58, 19, 132, 197]}else{bytes};
 assert_eq!(words[0],((u64::from(effective[0])) << 0) | ((u64::from(effective[1])) << 8) | ((u64::from(effective[2])) << 16) | ((u64::from(effective[3])) << 24) | ((u64::from(effective[4])) << 32) | ((u64::from(effective[5])) << 40) | ((u64::from(effective[6])) << 48) | ((u64::from(effective[7])) << 56));
 assert_eq!(words[1],((u64::from(effective[8])) << 0) | ((u64::from(effective[9])) << 8) | ((u64::from(effective[10])) << 16) | ((u64::from(effective[11])) << 24) | ((u64::from(effective[12])) << 32) | ((u64::from(effective[13])) << 40) | ((u64::from(effective[14])) << 48) | ((u64::from(effective[15])) << 56));
 assert_eq!(words[2],((u64::from(effective[16])) << 0) | ((u64::from(effective[17])) << 8) | ((u64::from(effective[18])) << 16) | ((u64::from(effective[19])) << 24) | ((u64::from(effective[20])) << 32) | ((u64::from(effective[21])) << 40) | ((u64::from(effective[22])) << 48) | ((u64::from(effective[23])) << 56));
 assert_eq!(words[3],((u64::from(effective[24])) << 0) | ((u64::from(effective[25])) << 8) | ((u64::from(effective[26])) << 16) | ((u64::from(effective[27])) << 24) | ((u64::from(effective[28])) << 32) | ((u64::from(effective[29])) << 40) | ((u64::from(effective[30])) << 48) | ((u64::from(effective[31])) << 56));
 assert_eq!(words[4],((u64::from(effective[32])) << 0) | ((u64::from(effective[33])) << 8) | ((u64::from(effective[34])) << 16) | ((u64::from(effective[35])) << 24) | ((u64::from(effective[36])) << 32) | ((u64::from(effective[37])) << 40) | ((u64::from(effective[38])) << 48) | ((u64::from(effective[39])) << 56));
 assert_eq!(words[5],((u64::from(effective[40])) << 0) | ((u64::from(effective[41])) << 8) | ((u64::from(effective[42])) << 16) | ((u64::from(effective[43])) << 24) | ((u64::from(effective[44])) << 32) | ((u64::from(effective[45])) << 40) | ((u64::from(effective[46])) << 48) | ((u64::from(effective[47])) << 56));
 assert_eq!(words[6],((u64::from(effective[48])) << 0) | ((u64::from(effective[49])) << 8) | ((u64::from(effective[50])) << 16) | ((u64::from(effective[51])) << 24) | ((u64::from(effective[52])) << 32) | ((u64::from(effective[53])) << 40) | ((u64::from(effective[54])) << 48) | ((u64::from(effective[55])) << 56));
 assert_eq!(words[7],((u64::from(effective[56])) << 0) | ((u64::from(effective[57])) << 8) | ((u64::from(effective[58])) << 16) | ((u64::from(effective[59])) << 24) | ((u64::from(effective[60])) << 32) | ((u64::from(effective[61])) << 40) | ((u64::from(effective[62])) << 48) | ((u64::from(effective[63])) << 56));
}
#[kani::proof]
fn original_xoshiro512starstar_from_seed_words(){
 let bytes:[u8;64]=kani::any();
 let rng=<crate::Xoshiro512StarStar as SeedableRng>::from_seed(crate::Seed512(bytes));
 let words=crate::xoshiro512starstar::proof_seed_words(&rng);
 let effective=if bytes==[0;64]{[175, 205, 29, 123, 57, 168, 32, 226, 244, 101, 185, 161, 106, 158, 120, 110, 79, 69, 9, 128, 24, 93, 196, 6, 236, 129, 76, 114, 168, 184, 139, 248, 155, 116, 168, 81, 106, 137, 57, 27, 234, 162, 126, 116, 12, 159, 203, 83, 225, 50, 69, 31, 190, 154, 130, 44, 60, 171, 22, 201, 58, 19, 132, 197]}else{bytes};
 assert_eq!(words[0],((u64::from(effective[0])) << 0) | ((u64::from(effective[1])) << 8) | ((u64::from(effective[2])) << 16) | ((u64::from(effective[3])) << 24) | ((u64::from(effective[4])) << 32) | ((u64::from(effective[5])) << 40) | ((u64::from(effective[6])) << 48) | ((u64::from(effective[7])) << 56));
 assert_eq!(words[1],((u64::from(effective[8])) << 0) | ((u64::from(effective[9])) << 8) | ((u64::from(effective[10])) << 16) | ((u64::from(effective[11])) << 24) | ((u64::from(effective[12])) << 32) | ((u64::from(effective[13])) << 40) | ((u64::from(effective[14])) << 48) | ((u64::from(effective[15])) << 56));
 assert_eq!(words[2],((u64::from(effective[16])) << 0) | ((u64::from(effective[17])) << 8) | ((u64::from(effective[18])) << 16) | ((u64::from(effective[19])) << 24) | ((u64::from(effective[20])) << 32) | ((u64::from(effective[21])) << 40) | ((u64::from(effective[22])) << 48) | ((u64::from(effective[23])) << 56));
 assert_eq!(words[3],((u64::from(effective[24])) << 0) | ((u64::from(effective[25])) << 8) | ((u64::from(effective[26])) << 16) | ((u64::from(effective[27])) << 24) | ((u64::from(effective[28])) << 32) | ((u64::from(effective[29])) << 40) | ((u64::from(effective[30])) << 48) | ((u64::from(effective[31])) << 56));
 assert_eq!(words[4],((u64::from(effective[32])) << 0) | ((u64::from(effective[33])) << 8) | ((u64::from(effective[34])) << 16) | ((u64::from(effective[35])) << 24) | ((u64::from(effective[36])) << 32) | ((u64::from(effective[37])) << 40) | ((u64::from(effective[38])) << 48) | ((u64::from(effective[39])) << 56));
 assert_eq!(words[5],((u64::from(effective[40])) << 0) | ((u64::from(effective[41])) << 8) | ((u64::from(effective[42])) << 16) | ((u64::from(effective[43])) << 24) | ((u64::from(effective[44])) << 32) | ((u64::from(effective[45])) << 40) | ((u64::from(effective[46])) << 48) | ((u64::from(effective[47])) << 56));
 assert_eq!(words[6],((u64::from(effective[48])) << 0) | ((u64::from(effective[49])) << 8) | ((u64::from(effective[50])) << 16) | ((u64::from(effective[51])) << 24) | ((u64::from(effective[52])) << 32) | ((u64::from(effective[53])) << 40) | ((u64::from(effective[54])) << 48) | ((u64::from(effective[55])) << 56));
 assert_eq!(words[7],((u64::from(effective[56])) << 0) | ((u64::from(effective[57])) << 8) | ((u64::from(effective[58])) << 16) | ((u64::from(effective[59])) << 24) | ((u64::from(effective[60])) << 32) | ((u64::from(effective[61])) << 40) | ((u64::from(effective[62])) << 48) | ((u64::from(effective[63])) << 56));
}
