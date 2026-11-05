use endian_type::{BigEndian,LittleEndian};
pub fn big_from<T>(value:BigEndian<T>)->BigEndian<T> { <BigEndian<T> as From<BigEndian<T>>>::from(value) }
pub fn big_borrow<T>(value:&BigEndian<T>)->&BigEndian<T> { <BigEndian<T> as core::borrow::Borrow<BigEndian<T>>>::borrow(value) }
pub fn big_borrow_mut<T>(value:&mut BigEndian<T>)->&mut BigEndian<T> { <BigEndian<T> as core::borrow::BorrowMut<BigEndian<T>>>::borrow_mut(value) }
pub fn little_from<T>(value:LittleEndian<T>)->LittleEndian<T> { <LittleEndian<T> as From<LittleEndian<T>>>::from(value) }
pub fn little_borrow<T>(value:&LittleEndian<T>)->&LittleEndian<T> { <LittleEndian<T> as core::borrow::Borrow<LittleEndian<T>>>::borrow(value) }
pub fn little_borrow_mut<T>(value:&mut LittleEndian<T>)->&mut LittleEndian<T> { <LittleEndian<T> as core::borrow::BorrowMut<LittleEndian<T>>>::borrow_mut(value) }
