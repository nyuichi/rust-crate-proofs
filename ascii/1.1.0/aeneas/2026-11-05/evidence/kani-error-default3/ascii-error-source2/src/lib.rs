use ascii::{ToAsciiChar,ToAsciiCharError};
use std::error::Error;
fn original_error() -> ToAsciiCharError {
 match <u8 as ToAsciiChar>::to_ascii_char(255) {
  Err(error)=>error,
  Ok(_)=>panic!("original checked conversion must reject 255"),
 }
}
#[kani::proof]
fn error_source_none_complete_unit_receiver() {
 let receiver=original_error();
 assert_eq!(core::mem::size_of::<ToAsciiCharError>(),0);
 assert!(<ToAsciiCharError as Error>::source(&receiver).is_none());
}
#[kani::proof]
#[allow(deprecated)]
fn error_cause_none_complete_unit_receiver() {
 let receiver=original_error();
 assert_eq!(core::mem::size_of::<ToAsciiCharError>(),0);
 assert!(<ToAsciiCharError as Error>::cause(&receiver).is_none());
}
