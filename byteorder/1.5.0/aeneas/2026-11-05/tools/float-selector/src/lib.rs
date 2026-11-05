use byteorder::{BigEndian,LittleEndian,ByteOrder};
pub fn be_write_f32(buf:&mut[u8],n:f32){BigEndian::write_f32(buf,n)}
pub fn be_write_f64(buf:&mut[u8],n:f64){BigEndian::write_f64(buf,n)}
pub fn le_write_f32(buf:&mut[u8],n:f32){LittleEndian::write_f32(buf,n)}
pub fn le_write_f64(buf:&mut[u8],n:f64){LittleEndian::write_f64(buf,n)}
