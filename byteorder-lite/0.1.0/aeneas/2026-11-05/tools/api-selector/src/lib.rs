use byteorder_lite::{BigEndian,LittleEndian,ByteOrder};
pub fn be_write_i16(buf: &mut[u8], n:i16) { BigEndian::write_i16(buf,n) }
pub fn be_write_i32(buf: &mut[u8], n:i32) { BigEndian::write_i32(buf,n) }
pub fn be_write_i64(buf: &mut[u8], n:i64) { BigEndian::write_i64(buf,n) }
pub fn be_write_i128(buf: &mut[u8], n:i128) { BigEndian::write_i128(buf,n) }
pub fn le_write_i16(buf: &mut[u8], n:i16) { LittleEndian::write_i16(buf,n) }
pub fn le_write_i32(buf: &mut[u8], n:i32) { LittleEndian::write_i32(buf,n) }
pub fn le_write_i64(buf: &mut[u8], n:i64) { LittleEndian::write_i64(buf,n) }
pub fn le_write_i128(buf: &mut[u8], n:i128) { LittleEndian::write_i128(buf,n) }
