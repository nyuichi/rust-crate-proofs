#[allow(unused_imports)] use rand_core::TryRng;
fn reference_xoroshiro64star(words:[u32;2])->(u32,[u32;2]){
 let [a,b]=words;
 let output=a.wrapping_mul(0x9E3779BB);
 let next={ let q=b^a; [a.rotate_left(26)^q^(q<<9),q.rotate_left(13)] };
 (output,next)
}
#[kani::proof]
fn original_xoroshiro64star_next64_matches_two_steps(){
 let words:[u32;2]=kani::any();
 let mut rng=crate::xoroshiro64star::proof_from_words(words);
 let (low,s1)=reference_xoroshiro64star(words);
 let (high,s2)=reference_xoroshiro64star(s1);
 let expected=(u64::from(high)<<32)|u64::from(low);
 let result=match rng.try_next_u64(){Ok(v)=>v,Err(e)=>match e{}};
 assert_eq!(result,expected);
 let actual=crate::xoroshiro64star::proof_words(&rng);
 assert_eq!(actual[0],s2[0]);
 assert_eq!(actual[1],s2[1]);
}

fn reference_xoroshiro64starstar(words:[u32;2])->(u32,[u32;2]){
 let [a,b]=words;
 let output=a.wrapping_mul(0x9E3779BB).rotate_left(5).wrapping_mul(5);
 let next={ let q=b^a; [a.rotate_left(26)^q^(q<<9),q.rotate_left(13)] };
 (output,next)
}
#[kani::proof]
fn original_xoroshiro64starstar_next64_matches_two_steps(){
 let words:[u32;2]=kani::any();
 let mut rng=crate::xoroshiro64starstar::proof_from_words(words);
 let (low,s1)=reference_xoroshiro64starstar(words);
 let (high,s2)=reference_xoroshiro64starstar(s1);
 let expected=(u64::from(high)<<32)|u64::from(low);
 let result=match rng.try_next_u64(){Ok(v)=>v,Err(e)=>match e{}};
 assert_eq!(result,expected);
 let actual=crate::xoroshiro64starstar::proof_words(&rng);
 assert_eq!(actual[0],s2[0]);
 assert_eq!(actual[1],s2[1]);
}

fn reference_xoshiro128plus(words:[u32;4])->(u32,[u32;4]){
 let [a,b,c,d]=words;
 let output=a.wrapping_add(d);
 let next={ let p=c^a;let q=d^b;[a^q,b^p,p^(b<<9),q.rotate_left(11)] };
 (output,next)
}
#[kani::proof]
fn original_xoshiro128plus_next64_matches_two_steps(){
 let words:[u32;4]=kani::any();
 let mut rng=crate::xoshiro128plus::proof_from_words(words);
 let (low,s1)=reference_xoshiro128plus(words);
 let (high,s2)=reference_xoshiro128plus(s1);
 let expected=(u64::from(high)<<32)|u64::from(low);
 let result=match rng.try_next_u64(){Ok(v)=>v,Err(e)=>match e{}};
 assert_eq!(result,expected);
 let actual=crate::xoshiro128plus::proof_words(&rng);
 assert_eq!(actual[0],s2[0]);
 assert_eq!(actual[1],s2[1]);
 assert_eq!(actual[2],s2[2]);
 assert_eq!(actual[3],s2[3]);
}

fn reference_xoshiro128plusplus(words:[u32;4])->(u32,[u32;4]){
 let [a,b,c,d]=words;
 let output=a.wrapping_add(d).rotate_left(7).wrapping_add(a);
 let next={ let p=c^a;let q=d^b;[a^q,b^p,p^(b<<9),q.rotate_left(11)] };
 (output,next)
}
#[kani::proof]
fn original_xoshiro128plusplus_next64_matches_two_steps(){
 let words:[u32;4]=kani::any();
 let mut rng=crate::xoshiro128plusplus::proof_from_words(words);
 let (low,s1)=reference_xoshiro128plusplus(words);
 let (high,s2)=reference_xoshiro128plusplus(s1);
 let expected=(u64::from(high)<<32)|u64::from(low);
 let result=match rng.try_next_u64(){Ok(v)=>v,Err(e)=>match e{}};
 assert_eq!(result,expected);
 let actual=crate::xoshiro128plusplus::proof_words(&rng);
 assert_eq!(actual[0],s2[0]);
 assert_eq!(actual[1],s2[1]);
 assert_eq!(actual[2],s2[2]);
 assert_eq!(actual[3],s2[3]);
}

fn reference_xoshiro128starstar(words:[u32;4])->(u32,[u32;4]){
 let [a,b,c,d]=words;
 let output=b.wrapping_mul(5).rotate_left(7).wrapping_mul(9);
 let next={ let p=c^a;let q=d^b;[a^q,b^p,p^(b<<9),q.rotate_left(11)] };
 (output,next)
}
#[kani::proof]
fn original_xoshiro128starstar_next64_matches_two_steps(){
 let words:[u32;4]=kani::any();
 let mut rng=crate::xoshiro128starstar::proof_from_words(words);
 let (low,s1)=reference_xoshiro128starstar(words);
 let (high,s2)=reference_xoshiro128starstar(s1);
 let expected=(u64::from(high)<<32)|u64::from(low);
 let result=match rng.try_next_u64(){Ok(v)=>v,Err(e)=>match e{}};
 assert_eq!(result,expected);
 let actual=crate::xoshiro128starstar::proof_words(&rng);
 assert_eq!(actual[0],s2[0]);
 assert_eq!(actual[1],s2[1]);
 assert_eq!(actual[2],s2[2]);
 assert_eq!(actual[3],s2[3]);
}
