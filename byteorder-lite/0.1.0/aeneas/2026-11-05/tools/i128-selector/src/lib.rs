use byteorder_lite::{ByteOrder,BigEndian,LittleEndian};
pub fn be_read_int128(buf:&[u8],count:usize)->i128 { BigEndian::read_int128(buf,count) }
pub fn le_read_int128(buf:&[u8],count:usize)->i128 { LittleEndian::read_int128(buf,count) }
pub fn be_write_int128(buf:&mut[u8],n:i128,count:usize) { BigEndian::write_int128(buf,n,count) }
pub fn le_write_int128(buf:&mut[u8],n:i128,count:usize) { LittleEndian::write_int128(buf,n,count) }
