use endian_type::{BigEndian,LittleEndian};
pub fn big_view<T:Sized+Copy>(x:&BigEndian<T>)->&[u8] { x.as_byte_slice() }
pub fn big_view_mut<T:Sized+Copy>(x:&mut BigEndian<T>)->&mut [u8] { x.as_byte_slice_mut() }
pub unsafe fn big_read<T:Sized+Copy>(x:&[u8])->BigEndian<T> { BigEndian::from_byte_slice(x) }
pub fn little_view<T:Sized+Copy>(x:&LittleEndian<T>)->&[u8] { x.as_byte_slice() }
pub fn little_view_mut<T:Sized+Copy>(x:&mut LittleEndian<T>)->&mut [u8] { x.as_byte_slice_mut() }
pub unsafe fn little_read<T:Sized+Copy>(x:&[u8])->LittleEndian<T> { LittleEndian::from_byte_slice(x) }
