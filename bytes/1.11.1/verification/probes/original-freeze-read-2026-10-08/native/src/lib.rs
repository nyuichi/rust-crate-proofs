#[cfg(test)]
mod tests {
 use bytes::BytesMut;
 #[test]
 fn original_allocation_write_freeze_read() {
  for input in [&b""[..],&b"a"[..],&b"original bytes allocation"[..]] {
   let mut value=BytesMut::with_capacity(input.len()+8);
   let pointer=value.as_ptr();
   value.extend_from_slice(input);
   assert_eq!(value.as_ptr(),pointer);
   let frozen=value.freeze();
   assert_eq!(frozen.as_ptr(),pointer);
   assert_eq!(frozen.as_ref(),input);
   // Actual original Drop executes here, but is not covered by the proof gate.
  }
 }
}
