// BEGIN EXACT BYTESMUT INVARIANT
#[cfg(all(creusot, bytes_proof_valid_handle))]
impl creusot_std::invariant::Invariant for BytesMut {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        pearlite! { self.proof_empty_valid() || (self.proof_unique_owned() && self.proof_initialized()) }
    }
}