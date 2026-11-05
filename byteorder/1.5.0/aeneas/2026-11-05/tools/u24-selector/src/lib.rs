use byteorder::{ByteOrder,BigEndian,LittleEndian};
pub fn be_read_u24(buf:&[u8])->u32 { BigEndian::read_u24(buf) }
pub fn le_read_u24(buf:&[u8])->u32 { LittleEndian::read_u24(buf) }
pub fn be_read_uint128(buf:&[u8],n:usize)->u128 { BigEndian::read_uint128(buf,n) }
pub fn le_read_uint128(buf:&[u8],n:usize)->u128 { LittleEndian::read_uint128(buf,n) }
