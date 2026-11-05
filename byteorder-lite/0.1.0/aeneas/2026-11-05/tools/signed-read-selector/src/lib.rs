use byteorder_lite::{ByteOrder,BigEndian,LittleEndian};
pub fn be_read_int(buf:&[u8],count:usize)->i64{BigEndian::read_int(buf,count)}
pub fn be_read_i24(buf:&[u8])->i32{BigEndian::read_i24(buf)}
pub fn be_read_uint128(buf:&[u8],count:usize)->u128{BigEndian::read_uint128(buf,count)}
pub fn le_read_int(buf:&[u8],count:usize)->i64{LittleEndian::read_int(buf,count)}
pub fn le_read_i24(buf:&[u8])->i32{LittleEndian::read_i24(buf)}
pub fn le_read_uint128(buf:&[u8],count:usize)->u128{LittleEndian::read_uint128(buf,count)}
