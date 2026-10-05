// u16::MAX is reserved as the PathAndQuery no-query sentinel.
pub(super) const MAX_LEN: usize = (u16::MAX - 1) as usize;
