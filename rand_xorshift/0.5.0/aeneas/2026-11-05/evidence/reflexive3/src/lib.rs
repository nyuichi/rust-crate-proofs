use rand_xorshift::XorShiftRng;
pub fn original_from(value:XorShiftRng)->XorShiftRng { <XorShiftRng as From<XorShiftRng>>::from(value) }
pub fn original_borrow(value:&XorShiftRng)->&XorShiftRng { <XorShiftRng as core::borrow::Borrow<XorShiftRng>>::borrow(value) }
pub fn original_borrow_mut(value:&mut XorShiftRng)->&mut XorShiftRng { <XorShiftRng as core::borrow::BorrowMut<XorShiftRng>>::borrow_mut(value) }
