use ascii::AsciiChar;
use core::cmp::Ordering;
fn original_input(code: u8) -> AsciiChar {
    // Exactly the complete domain of valid AsciiChar discriminants, not a
    // restriction on target API inputs: all 128 enum variants are represented.
    kani::assume(code < 128);
    let value = AsciiChar::new(char::from(code));
    assert_eq!(value.as_byte(), code);
    value
}
fn reference_order(left: u32, right: u32) -> Option<Ordering> {
 if left<right {Some(Ordering::Less)} else if left>right {Some(Ordering::Greater)} else {Some(Ordering::Equal)}
}
fn original_unicode(code: u32) -> char {
 kani::assume(code<=0x10ffff && !(0xd800..=0xdfff).contains(&code));
 char::from_u32(code).unwrap()
}
#[kani::proof]
fn ne_u8_forward(){let code:u8=kani::any();let value=original_input(code);let other:u8=kani::any();let other_code=u32::from(other);assert_eq!(<AsciiChar as PartialEq<u8>>::ne(&value,&other),u32::from(code)!=other_code);}
#[kani::proof]
fn lt_u8_forward(){let code:u8=kani::any();let value=original_input(code);let other:u8=kani::any();let other_code=u32::from(other);assert_eq!(<AsciiChar as PartialOrd<u8>>::lt(&value,&other),u32::from(code)<other_code);}
#[kani::proof]
fn le_u8_forward(){let code:u8=kani::any();let value=original_input(code);let other:u8=kani::any();let other_code=u32::from(other);assert_eq!(<AsciiChar as PartialOrd<u8>>::le(&value,&other),u32::from(code)<=other_code);}
#[kani::proof]
fn gt_u8_forward(){let code:u8=kani::any();let value=original_input(code);let other:u8=kani::any();let other_code=u32::from(other);assert_eq!(<AsciiChar as PartialOrd<u8>>::gt(&value,&other),u32::from(code)>other_code);}
#[kani::proof]
fn ge_u8_forward(){let code:u8=kani::any();let value=original_input(code);let other:u8=kani::any();let other_code=u32::from(other);assert_eq!(<AsciiChar as PartialOrd<u8>>::ge(&value,&other),u32::from(code)>=other_code);}
#[kani::proof]
fn ne_u8_reverse(){let code:u8=kani::any();let value=original_input(code);let other:u8=kani::any();let other_code=u32::from(other);assert_eq!(<u8 as PartialEq<AsciiChar>>::ne(&other,&value),other_code!=u32::from(code));}
#[kani::proof]
fn lt_u8_reverse(){let code:u8=kani::any();let value=original_input(code);let other:u8=kani::any();let other_code=u32::from(other);assert_eq!(<u8 as PartialOrd<AsciiChar>>::lt(&other,&value),other_code<u32::from(code));}
#[kani::proof]
fn le_u8_reverse(){let code:u8=kani::any();let value=original_input(code);let other:u8=kani::any();let other_code=u32::from(other);assert_eq!(<u8 as PartialOrd<AsciiChar>>::le(&other,&value),other_code<=u32::from(code));}
#[kani::proof]
fn gt_u8_reverse(){let code:u8=kani::any();let value=original_input(code);let other:u8=kani::any();let other_code=u32::from(other);assert_eq!(<u8 as PartialOrd<AsciiChar>>::gt(&other,&value),other_code>u32::from(code));}
#[kani::proof]
fn ge_u8_reverse(){let code:u8=kani::any();let value=original_input(code);let other:u8=kani::any();let other_code=u32::from(other);assert_eq!(<u8 as PartialOrd<AsciiChar>>::ge(&other,&value),other_code>=u32::from(code));}
#[kani::proof]
fn ne_char_forward(){let code:u8=kani::any();let value=original_input(code);let other_code:u32=kani::any();let other=original_unicode(other_code);assert_eq!(<AsciiChar as PartialEq<char>>::ne(&value,&other),u32::from(code)!=other_code);}
#[kani::proof]
fn lt_char_forward(){let code:u8=kani::any();let value=original_input(code);let other_code:u32=kani::any();let other=original_unicode(other_code);assert_eq!(<AsciiChar as PartialOrd<char>>::lt(&value,&other),u32::from(code)<other_code);}
#[kani::proof]
fn le_char_forward(){let code:u8=kani::any();let value=original_input(code);let other_code:u32=kani::any();let other=original_unicode(other_code);assert_eq!(<AsciiChar as PartialOrd<char>>::le(&value,&other),u32::from(code)<=other_code);}
#[kani::proof]
fn gt_char_forward(){let code:u8=kani::any();let value=original_input(code);let other_code:u32=kani::any();let other=original_unicode(other_code);assert_eq!(<AsciiChar as PartialOrd<char>>::gt(&value,&other),u32::from(code)>other_code);}
#[kani::proof]
fn ge_char_forward(){let code:u8=kani::any();let value=original_input(code);let other_code:u32=kani::any();let other=original_unicode(other_code);assert_eq!(<AsciiChar as PartialOrd<char>>::ge(&value,&other),u32::from(code)>=other_code);}
#[kani::proof]
fn ne_char_reverse(){let code:u8=kani::any();let value=original_input(code);let other_code:u32=kani::any();let other=original_unicode(other_code);assert_eq!(<char as PartialEq<AsciiChar>>::ne(&other,&value),other_code!=u32::from(code));}
#[kani::proof]
fn lt_char_reverse(){let code:u8=kani::any();let value=original_input(code);let other_code:u32=kani::any();let other=original_unicode(other_code);assert_eq!(<char as PartialOrd<AsciiChar>>::lt(&other,&value),other_code<u32::from(code));}
#[kani::proof]
fn le_char_reverse(){let code:u8=kani::any();let value=original_input(code);let other_code:u32=kani::any();let other=original_unicode(other_code);assert_eq!(<char as PartialOrd<AsciiChar>>::le(&other,&value),other_code<=u32::from(code));}
#[kani::proof]
fn gt_char_reverse(){let code:u8=kani::any();let value=original_input(code);let other_code:u32=kani::any();let other=original_unicode(other_code);assert_eq!(<char as PartialOrd<AsciiChar>>::gt(&other,&value),other_code>u32::from(code));}
#[kani::proof]
fn ge_char_reverse(){let code:u8=kani::any();let value=original_input(code);let other_code:u32=kani::any();let other=original_unicode(other_code);assert_eq!(<char as PartialOrd<AsciiChar>>::ge(&other,&value),other_code>=u32::from(code));}
