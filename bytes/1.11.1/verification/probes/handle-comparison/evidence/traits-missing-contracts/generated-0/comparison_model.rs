
#[cfg(creusot)]
impl BytesMut {
    // Total outside the invariant too: Unknown/absent slots map to zero, but
    // this assigns no ownership or initialization. Valid handles prove Known.
    #[logic]
    pub fn proof_byte_model(self) -> Seq<Int> {
        pearlite! { Seq::create(self.len@, |index| match self.proof_view_slot(index) {
            Some(Some(byte)) => byte@,
            _ => 0int,
        }) }
    }
}
#[cfg(creusot)]
impl DeepModel for BytesMut {
    type DeepModelTy = Seq<Int>;
    #[logic]
    fn deep_model(self) -> Seq<Int> { self.proof_byte_model() }
}
