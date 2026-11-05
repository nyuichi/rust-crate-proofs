use byteorder_lite::{ByteOrder,BigEndian,LittleEndian};
pub fn be_read_u48(buf:&[u8])->u64 { BigEndian::read_u48(buf) }
pub fn be_write_u48(buf:&mut[u8],n:u64) { BigEndian::write_u48(buf,n) }
pub fn le_read_u48(buf:&[u8])->u64 { LittleEndian::read_u48(buf) }
pub fn le_write_u48(buf:&mut[u8],n:u64) { LittleEndian::write_u48(buf,n) }
pub fn be_read_i48(buf:&[u8])->i64 { BigEndian::read_i48(buf) }
pub fn be_write_i48(buf:&mut[u8],n:i64) { BigEndian::write_i48(buf,n) }
pub fn le_read_i48(buf:&[u8])->i64 { LittleEndian::read_i48(buf) }
pub fn le_write_i48(buf:&mut[u8],n:i64) { LittleEndian::write_i48(buf,n) }
