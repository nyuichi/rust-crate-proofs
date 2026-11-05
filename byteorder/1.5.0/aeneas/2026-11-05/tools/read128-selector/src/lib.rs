use byteorder::{ByteOrder,BigEndian,LittleEndian};
pub fn be_read_int128(buf:&[u8],count:usize)->i128 { BigEndian::read_int128(buf,count) }
pub fn le_read_int128(buf:&[u8],count:usize)->i128 { LittleEndian::read_int128(buf,count) }
