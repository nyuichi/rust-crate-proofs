use bytes_cursor_closure_native::cursor_scope;

#[test]
fn advances_preserve_suffix_and_owned_empty_drop() {
    let runs: &[&[usize]] = &[&[], &[0], &[1], &[usize::MAX], &[1,0,1,2,0,usize::MAX,7], &[usize::MAX,usize::MAX,0], &[2,3,5]];
    for len in [1usize, 2, 7, 31] {
        let expected: Vec<u8> = (0..len).map(|i| ((i * 37 + 11) % 251) as u8).collect();
        for (a,b) in [(0,len),(len/2,len),(0,0),(len,len),(len/2,len/2)] {
            for steps in runs {
                let mut consumed = 0;
                for by in *steps { consumed += core::cmp::min(*by,b-a-consumed); }
                assert_eq!(cursor_scope(expected.clone().into_boxed_slice(),a,b,steps), expected[a+consumed..b]);
            }
        }
    }
}
