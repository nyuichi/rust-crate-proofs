//! Reusable scoped immutable callbacks over the actual detached allocation.
//! The higher-ranked input lifetime prevents results from retaining that input
//! borrow. Callback behavior is specified by each function's own FnOnce contract.
use super::*;

/// Runs two callbacks on the same immutable bytes in real scoped threads, then
/// explicitly reclaims the allocation. Results may own data, but cannot borrow
/// the callback's input slice. Panic/unwind paths are outside this normal-return
/// contract.
#[requires(forall<arg: &[u8]> arg@ == input@ ==> first.precondition((arg,)))]
#[requires(forall<arg: &[u8]> arg@ == input@ ==> second.precondition((arg,)))]
#[ensures(exists<arg: &[u8]> arg@ == input@ && first.postcondition_once((arg,),result.0))]
#[ensures(exists<arg: &[u8]> arg@ == input@ && second.postcondition_once((arg,),result.1))]
pub fn with_shared_read<F,G,R,S>(input:Vec<u8>,first:F,second:G)->(R,S)
where
    F: Send + for<'a> FnOnce(&'a [u8])->R,
    G: Send + for<'a> FnOnce(&'a [u8])->S,
    R: Send,
    S: Send,
{
    use creusot_std::std::thread::{self,JoinHandleExt};
    let mut owner=Owner::new(input);
    let (left,right,context)=owner.share_pair();
    let parent=&context;
    let (left,right)=thread::scope(move |scope| {
        let left=scope.spawn(move |tokens| {
            let result=first(left.bytes);
            (result,left.close(parent,tokens))
        });
        let right=scope.spawn(move |tokens| {
            let result=second(right.bytes);
            (result,right.close(parent,tokens))
        });
        (left.join_unwrap(),right.join_unwrap())
    });
    let flags=owner.close(context,left.1,right.1);
    proof_assert!(flags.0 != flags.1);
    (left.0,right.0)
}

/// A concrete verified client of the generic callback API.
#[ensures(result.0@ == input@.len())]
#[ensures(result.1 == if input@.len() == 0 {None} else {Some(input@[0])})]
pub fn callback_length_and_first(input:Vec<u8>)->(usize,Option<u8>) {
    with_shared_read(input,
        |bytes:&[u8]| bytes.len(),
        |bytes:&[u8]| if bytes.len() == 0 {None} else {Some(bytes[0])})
}

#[cfg(all(test,not(creusot)))]
mod tests {
    use super::*;
    #[test]
    fn callbacks_return_owned_results_and_use_captures() {
        for len in [0,1,5] { for spare in [0,8] {
            let mut input=Vec::with_capacity(len+spare);
            input.extend((0..len).map(|i| (151+i) as u8));
            let expected=input.clone();
            let (a,b)=callback_length_and_first(input.clone());
            assert_eq!(a,len);assert_eq!(b,expected.first().copied());
            let owned=alloc::string::String::from("owned result");
            let (copy,(size,label))=with_shared_read(input,
                |bytes:&[u8]| bytes.to_vec(),
                move |bytes:&[u8]| (bytes.len(),owned));
            assert_eq!(copy,expected);assert_eq!(size,len);assert_eq!(label,"owned result");
        }}
    }
}
