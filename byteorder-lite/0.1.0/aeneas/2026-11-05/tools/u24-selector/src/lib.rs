use byteorder_lite::{ByteOrder,BigEndian,LittleEndian};
pub fn be_read_u24(buf:&[u8])->u32 { BigEndian::read_u24(buf) }
pub fn be_write_u24(buf:&mut[u8],n:u32) { BigEndian::write_u24(buf,n) }
pub fn le_read_u24(buf:&[u8])->u32 { LittleEndian::read_u24(buf) }
pub fn le_write_u24(buf:&mut[u8],n:u32) { LittleEndian::write_u24(buf,n) }
pub fn be_read_uint128(buf:&[u8],n:usize)->u128 { BigEndian::read_uint128(buf,n) }
pub fn le_read_uint128(buf:&[u8],n:usize)->u128 { LittleEndian::read_uint128(buf,n) }
