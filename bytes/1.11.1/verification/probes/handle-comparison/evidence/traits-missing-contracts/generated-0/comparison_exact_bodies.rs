pub fn __creusot_eq_bytes_mut(&self, other: &BytesMut) -> bool {
        crate::comparison_ops::equal(self.as_slice(), other.as_slice())
    }
pub fn __creusot_cmp_bytes_mut(&self, other: &BytesMut) -> cmp::Ordering {
        crate::comparison_ops::compare(self.as_slice(), other.as_slice())
    }
pub fn __creusot_eq_slice(&self, other: &[u8]) -> bool {
        crate::comparison_ops::equal(self.as_slice(), other)
    }
pub fn __creusot_cmp_slice(&self, other: &[u8]) -> cmp::Ordering {
        crate::comparison_ops::compare(self.as_slice(), other)
    }
impl PartialEq for BytesMut {
    fn eq(&self, other: &BytesMut) -> bool {
        crate::comparison_ops::equal(self.as_slice(), other.as_slice())
    }
}
impl PartialOrd for BytesMut {
    fn partial_cmp(&self, other: &BytesMut) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for BytesMut {
    fn cmp(&self, other: &BytesMut) -> cmp::Ordering {
        crate::comparison_ops::compare(self.as_slice(), other.as_slice())
    }
}
impl Eq for BytesMut {}
