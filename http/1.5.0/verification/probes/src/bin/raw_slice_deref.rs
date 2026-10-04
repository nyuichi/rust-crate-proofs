#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;

fn raw_slice_index<T>(slice: &mut [T], index: usize) -> &mut T {
    let pointer = slice as *mut [T];
    unsafe { &mut (*pointer)[index] }
}

fn main() {}
