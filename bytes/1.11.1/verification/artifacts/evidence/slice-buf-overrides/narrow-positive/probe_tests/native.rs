use bytes_slice_buf_overrides::{try_get_u8, try_get_u16, try_get_u32};
#[test]
fn checked_slice_overrides_preserve_short_input_and_error_metadata() {
    for len in 0..6 {
        let bytes = [1, 2, 3, 4, 5, 6];
        let original = &bytes[..len];
        let mut input = original;
        match try_get_u8(&mut input) {
            Ok(v) => { assert_eq!(v, 1); assert_eq!(input, &original[1..]); }
            Err(e) => { assert_eq!((e.requested,e.available),(1,len)); assert_eq!(input,original); }
        }
        let mut input = original;
        match try_get_u16(&mut input) {
            Ok(v) => { assert_eq!(v, 0x0102); assert_eq!(input, &original[2..]); }
            Err(e) => { assert_eq!((e.requested,e.available),(2,len)); assert_eq!(input,original); }
        }
        let mut input = original;
        match try_get_u32(&mut input) {
            Ok(v) => { assert_eq!(v, 0x01020304); assert_eq!(input, &original[4..]); }
            Err(e) => { assert_eq!((e.requested,e.available),(4,len)); assert_eq!(input,original); }
        }
    }
}
