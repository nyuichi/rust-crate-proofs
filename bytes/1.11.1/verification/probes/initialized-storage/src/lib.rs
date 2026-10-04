//! Probes initialized `Box<[u8]>` mutation through the real allocation.
//! This does not model Bytes/BytesMut aliasing or partially initialized capacity.
#![allow(unexpected_cfgs)]

use creusot_std::{ghost::perm::Perm, prelude::*};

#[path = "../../../../src/slice_mut_ops.rs"]
mod slice_mut_ops;

#[requires(count@ <= input@.len())]
#[ensures(result@ == input@[0..count@])]
#[ensures((^input)@ == input@[count@..])]
pub fn advance_initialized_slice<'a>(input: &mut &'a mut [u8], count: usize) -> &'a mut [u8] {
    slice_mut_ops::advance_slice_mut(input, count)
}

#[requires(count@ <= input@.len())]
#[ensures(result@ == input@[0..count@])]
#[ensures((^input)@ == input@[count@..])]
pub fn advance_maybe_uninit_slice<'a>(
    input: &mut &'a mut [core::mem::MaybeUninit<u8>],
    count: usize,
) -> &'a mut [core::mem::MaybeUninit<u8>] {
    slice_mut_ops::advance_slice_mut(input, count)
}

#[requires(source@.len() <= input@.len())]
#[ensures(result@ == source@)]
#[ensures((^input)@ == input@[source@.len()..])]
pub fn copy_initialized_prefix<'a>(input: &mut &'a mut [u8], source: &[u8]) -> &'a mut [u8] {
    slice_mut_ops::copy_to_slice_mut(input, source)
}

#[requires(count@ <= input@.len())]
#[ensures(result@.len() == count@)]
#[ensures(forall<i> 0 <= i && i < count@ ==> result@[i] == value)]
#[ensures((^input)@ == input@[count@..])]
pub fn fill_initialized_prefix<'a>(input: &mut &'a mut [u8], count: usize, value: u8) -> &'a mut [u8] {
    slice_mut_ops::fill_slice_mut(input, count, value)
}

/// Writes one arbitrary byte through the unique permission from the input Box,
/// then recovers the same allocation as a Box.
#[requires(index@ < input@.len())]
#[ensures(result@.len() == input@.len())]
#[ensures(result@[index@] == value)]
#[ensures(forall<i> 0 <= i && i < input@.len() && i != index@ ==> result@[i] == input@[i])]
pub fn write_one(input: Box<[u8]>, index: usize, value: u8) -> Box<[u8]> {
    let (pointer, mut permission) = Perm::from_box(input);
    let bytes = unsafe { Perm::as_mut(pointer, ghost! { &mut **permission }) };
    bytes[index] = value;
    unsafe { Perm::to_box(pointer, permission) }
}

/// Reads the written byte from the recovered Box and returns both observations.
#[requires(index@ < input@.len())]
#[ensures(result.0 == value)]
#[ensures(result.1@.len() == input@.len())]
#[ensures(result.1@[index@] == value)]
#[ensures(forall<i> 0 <= i && i < input@.len() && i != index@ ==> result.1@[i] == input@[i])]
pub fn write_read_recover(input: Box<[u8]>, index: usize, value: u8) -> (u8, Box<[u8]>) {
    let (pointer, mut permission) = Perm::from_box(input);
    let bytes = unsafe { Perm::as_mut(pointer, ghost! { &mut **permission }) };
    bytes[index] = value;
    let recovered = unsafe { Perm::to_box(pointer, permission) };
    let observed = recovered[index];
    (observed, recovered)
}

/// Copies an arbitrary initialized source slice into a bounded destination
/// window and recovers the original boxed allocation. Prefix and suffix bytes
/// are preserved.
#[requires(start@ <= input@.len())]
#[requires(source@.len() <= input@.len() - start@)]
#[ensures(result@.len() == input@.len())]
#[ensures(result@.subsequence(0, start@) == input@.subsequence(0, start@))]
#[ensures(result@.subsequence(start@, start@ + source@.len()) == source@)]
#[ensures(result@.subsequence(start@ + source@.len(), input@.len()) ==
          input@.subsequence(start@ + source@.len(), input@.len()))]
pub fn copy_window(input: Box<[u8]>, start: usize, source: &[u8]) -> Box<[u8]> {
    let (pointer, mut permission) = Perm::from_box(input);
    let bytes = unsafe { Perm::as_mut(pointer, ghost! { &mut **permission }) };
    let (prefix, tail) = bytes.split_at_mut(start);
    let (window, _suffix) = tail.split_at_mut(source.len());
    window.copy_from_slice(source);
    let _ = prefix;
    unsafe { Perm::to_box(pointer, permission) }
}

/// Exposes the suffix of a window represented by a concatenation.
#[logic]
#[requires(0 <= start)]
#[requires(whole.len() == start + prefix.len() + suffix.len())]
#[requires(whole[start..] == prefix.concat(suffix))]
#[ensures(whole[start + prefix.len()..] == suffix)]
fn suffix_of_window(whole: Seq<u8>, start: Int, prefix: Seq<u8>, suffix: Seq<u8>) {
    // Name the nested slice index so the subsequence and concatenation facts
    // can be applied before extensional equality closes the result.
    proof_assert!(forall<i> 0 <= i && i < suffix.len() ==>
        whole[start..][prefix.len() + i] == suffix[i]);
    proof_assert!(forall<i> 0 <= i && i < suffix.len() ==>
        whole[start + prefix.len() + i] == whole[start..][prefix.len() + i]);
    proof_assert!(forall<i> 0 <= i && i < suffix.len() ==>
        whole[start + prefix.len()..][i] == suffix[i]);
}

/// Connects the safe initialized-slice copy helper to a real boxed allocation
/// owned by `Perm`, then recovers that same allocation.
#[requires(start@ <= input@.len())]
#[requires(source@.len() <= input@.len() - start@)]
#[ensures(result@.len() == input@.len())]
#[ensures(result@.subsequence(0, start@) == input@.subsequence(0, start@))]
#[ensures(result@.subsequence(start@, start@ + source@.len()) == source@)]
#[ensures(result@.subsequence(start@ + source@.len(), input@.len()) ==
          input@.subsequence(start@ + source@.len(), input@.len()))]
pub fn boxed_copy_window(input: Box<[u8]>, start: usize, source: &[u8]) -> Box<[u8]> {
    let original = snapshot!(input@);
    let (pointer, mut permission) = Perm::from_box(input);
    {
        let bytes = unsafe { Perm::as_mut(pointer, ghost! { &mut **permission }) };
        let (_prefix, tail) = bytes.split_at_mut(start);
        let mut cursor = tail;
        let written = slice_mut_ops::copy_to_slice_mut(&mut cursor, source);
        proof_assert!(written@ == source@);
        proof_assert!(cursor@ == original[start@ + source@.len()..]);
        let _ = written;
    }
    let recovered = unsafe { Perm::to_box(pointer, permission) };
    proof_assert!({
        suffix_of_window(recovered@, start@, source@, original[start@ + source@.len()..]);
        true
    });
    recovered
}

/// Mutates independently borrowed halves of one boxed slice, demonstrating
/// that disjoint initialized regions can both be written before recovery.
#[requires(input@.len() >= 2)]
#[ensures(result@.len() == input@.len())]
#[ensures(result@[0] == left)]
#[ensures(result@[input@.len() - 1] == right)]
#[ensures(forall<i> 0 < i && i < input@.len() - 1 ==> result@[i] == input@[i])]
pub fn write_disjoint_ends(input: Box<[u8]>, left: u8, right: u8) -> Box<[u8]> {
    let (pointer, mut permission) = Perm::from_box(input);
    let bytes = unsafe { Perm::as_mut(pointer, ghost! { &mut **permission }) };
    let (first, rest) = bytes.split_at_mut(1);
    let (middle, last) = rest.split_at_mut(rest.len() - 1);
    first[0] = left;
    last[0] = right;
    let _ = middle;
    unsafe { Perm::to_box(pointer, permission) }
}

#[cfg(feature = "wrong_byte")]
#[requires(index@ < input@.len())]
#[requires(value < u8::MAX)]
#[ensures(result@.len() == input@.len())]
#[ensures(result@[index@]@ == value@ + 1)]
pub fn wrong_write_contract(input: Box<[u8]>, index: usize, value: u8) -> Box<[u8]> {
    write_one(input, index, value)
}

#[cfg(feature = "wrong_ownership")]
pub unsafe fn recover_without_ownership(pointer: *mut [u8]) -> Box<[u8]> {
    // Must fail: this pointer has no unique Perm/Box ownership token.
    Box::from_raw(pointer)
}
