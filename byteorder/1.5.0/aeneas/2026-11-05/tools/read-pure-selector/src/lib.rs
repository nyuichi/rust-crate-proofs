use byteorder::{ByteOrder,BigEndian,LittleEndian};
pub fn be_read_u48(buf:&[u8])->u64 { BigEndian::read_u48(buf) }
pub fn be_read_i24(buf:&[u8])->i32 { BigEndian::read_i24(buf) }
pub fn be_read_i48(buf:&[u8])->i64 { BigEndian::read_i48(buf) }
pub fn be_read_int(buf:&[u8],count:usize)->i64 { BigEndian::read_int(buf,count) }
pub fn le_read_u48(buf:&[u8])->u64 { LittleEndian::read_u48(buf) }
pub fn le_read_i24(buf:&[u8])->i32 { LittleEndian::read_i24(buf) }
pub fn le_read_i48(buf:&[u8])->i64 { LittleEndian::read_i48(buf) }
pub fn le_read_int(buf:&[u8],count:usize)->i64 { LittleEndian::read_int(buf,count) }
