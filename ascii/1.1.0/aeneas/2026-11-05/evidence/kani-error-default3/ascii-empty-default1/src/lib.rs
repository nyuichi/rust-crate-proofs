use ascii::AsciiStr;
#[kani::proof]
fn original_default_static_ascii_str_empty() {
 let result: &'static AsciiStr=<&'static AsciiStr as Default>::default();
 assert_eq!(result.len(),0);
 assert!(result.is_empty());
 assert_eq!(result.as_slice().len(),0);
 assert_eq!(result.as_bytes(),b"");
}
