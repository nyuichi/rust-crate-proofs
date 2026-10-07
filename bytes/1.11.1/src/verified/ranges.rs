//! Scoped callbacks on checked immutable ranges. Every callback still owns a
//! complete lifetime lease, even when its selected byte range is empty.
use super::*;
use core::ops::Range;

#[logic(open)]
pub fn range_fits(bytes:Seq<u8>,range:Range<usize>)->bool {
    pearlite! { range.start <= range.end && range.end@ <= bytes.len() }
}

/// Runs callbacks on two checked subranges of the shared physical bytes.
/// Invalid bounds return None after explicit allocation cleanup. Callback
/// results cannot retain either borrowed range because their input lifetime is
/// higher-ranked and their output type is independent of it.
#[requires(range_fits(input@,first_range) && range_fits(input@,second_range) ==>
    forall<arg:&[u8]> arg@ == input@[first_range.start@..first_range.end@] ==>
        first.precondition((arg,)))]
#[requires(range_fits(input@,first_range) && range_fits(input@,second_range) ==>
    forall<arg:&[u8]> arg@ == input@[second_range.start@..second_range.end@] ==>
        second.precondition((arg,)))]
#[ensures((result != None) == (range_fits(input@,first_range) && range_fits(input@,second_range)))]
#[ensures(result != None ==> exists<arg:&[u8]>
    arg@ == input@[first_range.start@..first_range.end@] &&
        first.postcondition_once((arg,),result.unwrap_logic().0))]
#[ensures(result != None ==> exists<arg:&[u8]>
    arg@ == input@[second_range.start@..second_range.end@] &&
        second.postcondition_once((arg,),result.unwrap_logic().1))]
pub fn with_shared_ranges<F,G,R,S>(input:Vec<u8>,first_range:Range<usize>,second_range:Range<usize>,
    first:F,second:G)->Option<(R,S)>
where
    F:Send + for<'a> FnOnce(&'a[u8])->R,
    G:Send + for<'a> FnOnce(&'a[u8])->S,
    R:Send,
    S:Send,
{
    use creusot_std::std::thread::{self,JoinHandleExt};
    let len=input.len();
    if first_range.start > first_range.end || first_range.end > len ||
        second_range.start > second_range.end || second_range.end > len {
        exclusive::ExclusiveBytes::from_vec(input).close();
        return None;
    }
    let mut owner=Owner::new(input);
    let (left,right,context)=owner.share_pair();
    let parent=&context;
    let (left,right)=thread::scope(move |scope| {
        let left=scope.spawn(move |tokens| {
            let result=first(&left.bytes[first_range]);
            (result,left.close(parent,tokens))
        });
        let right=scope.spawn(move |tokens| {
            let result=second(&right.bytes[second_range]);
            (result,right.close(parent,tokens))
        });
        (left.join_unwrap(),right.join_unwrap())
    });
    let flags=owner.close(context,left.1,right.1);
    proof_assert!(flags.0 != flags.1);
    Some((left.0,right.0))
}

/// A verified client observing actual first bytes of both selected ranges.
#[ensures((result != None) == (range_fits(input@,first) && range_fits(input@,second)))]
#[ensures(result != None ==> result.unwrap_logic().0 ==
    if first.start == first.end {None} else {Some(input@[first.start@])})]
#[ensures(result != None ==> result.unwrap_logic().1 ==
    if second.start == second.end {None} else {Some(input@[second.start@])})]
pub fn range_first_bytes(input:Vec<u8>,first:Range<usize>,second:Range<usize>)
    ->Option<(Option<u8>,Option<u8>)>
{
    with_shared_ranges(input,first,second,
        |bytes:&[u8]| if bytes.len() == 0 {None} else {Some(bytes[0])},
        |bytes:&[u8]| if bytes.len() == 0 {None} else {Some(bytes[0])})
}

#[cfg(all(test,not(creusot)))]
mod tests {
    use super::*;
    #[test]
    fn checked_ranges_return_owned_results_and_close_empty_leases() {
        let ranges=[0..0,0..2,1..4,5..5,2..1,0..6];
        for len in [0,1,5] {for spare in [0,8] {
            for first in &ranges {for second in &ranges {
                let mut input=Vec::with_capacity(len+spare);
                input.extend((0..len).map(|i| (191+i) as u8));
                let expected=match (input.get(first.clone()),input.get(second.clone())) {
                    (Some(a),Some(b))=>Some((a.to_vec(),b.to_vec())), _=>None,
                };
                let result=with_shared_ranges(input.clone(),first.clone(),second.clone(),
                    |bytes:&[u8]| bytes.to_vec(),|bytes:&[u8]| bytes.to_vec());
                assert_eq!(result,expected);
                let expected=expected.map(|(a,b)|(a.first().copied(),b.first().copied()));
                assert_eq!(range_first_bytes(input,first.clone(),second.clone()),expected);
            }}
        }}
    }
}
