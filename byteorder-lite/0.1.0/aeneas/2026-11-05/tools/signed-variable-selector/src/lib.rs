use byteorder_lite::{ByteOrder,BigEndian,LittleEndian};
pub fn be_write_i24(buf:&mut[u8],n:i32){BigEndian::write_i24(buf,n)}
pub fn be_write_int(buf:&mut[u8],n:i64,count:usize){BigEndian::write_int(buf,n,count)}
pub fn le_write_i24(buf:&mut[u8],n:i32){LittleEndian::write_i24(buf,n)}
pub fn le_write_int(buf:&mut[u8],n:i64,count:usize){LittleEndian::write_int(buf,n,count)}
