/// An error in parsing a chunk size.
// Note: Move this into the error enum once v2.0 is released.
#[cfg_attr(not(creusot), derive(Debug, PartialEq, Eq))]
pub struct InvalidChunkSize;
