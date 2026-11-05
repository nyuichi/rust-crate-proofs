use endian_type::{BigEndian,LittleEndian};
pub fn big_into<T,U:From<BigEndian<T>>>(x:BigEndian<T>)->U { <BigEndian<T> as Into<U>>::into(x) }
pub fn big_try_into<T,U:TryFrom<BigEndian<T>>>(x:BigEndian<T>)->Result<U,U::Error> { <BigEndian<T> as TryInto<U>>::try_into(x) }
pub fn big_try_from<T,U:Into<BigEndian<T>>>(x:U)->Result<BigEndian<T>,core::convert::Infallible> { <BigEndian<T> as TryFrom<U>>::try_from(x) }
pub fn little_into<T,U:From<LittleEndian<T>>>(x:LittleEndian<T>)->U { <LittleEndian<T> as Into<U>>::into(x) }
pub fn little_try_into<T,U:TryFrom<LittleEndian<T>>>(x:LittleEndian<T>)->Result<U,U::Error> { <LittleEndian<T> as TryInto<U>>::try_into(x) }
pub fn little_try_from<T,U:Into<LittleEndian<T>>>(x:U)->Result<LittleEndian<T>,core::convert::Infallible> { <LittleEndian<T> as TryFrom<U>>::try_from(x) }
