pub fn shared(s: &ascii::AsciiStr) -> &[ascii::AsciiChar] {
 <&[ascii::AsciiChar] as core::convert::From<&ascii::AsciiStr>>::from(s)
}
pub fn mutable(s: &mut ascii::AsciiStr) -> &mut [ascii::AsciiChar] {
 <&mut [ascii::AsciiChar] as core::convert::From<&mut ascii::AsciiStr>>::from(s)
}
