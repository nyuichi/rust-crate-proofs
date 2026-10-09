use bytes_shared_slice_views_native::nested_slice_scope;

#[test]
fn nested_ranges_preserve_exact_contents() {
    for len in [1usize, 2, 7, 31] {
        let expected: Vec<u8> = (0..len).map(|i| ((i * 37 + 11) % 251) as u8).collect();
        for (a,b) in [(0,len), (len/2,len), (0,0), (len,len), (len/2,len/2)] {
            let n=b-a;
            for (c,d) in [(0,n),(n/2,n),(0,0),(n,n),(n/2,n/2)] {
                assert_eq!(nested_slice_scope(expected.clone().into_boxed_slice(),a,b,c,d),expected[a+c..a+d]);
            }
        }
    }
}
