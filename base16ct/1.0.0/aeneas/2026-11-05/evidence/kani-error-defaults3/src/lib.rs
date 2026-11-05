#![allow(deprecated)]
#[cfg(kani)] fn arbitrary_error()->base16ct::Error {if kani::any(){base16ct::Error::InvalidEncoding}else{base16ct::Error::InvalidLength}}
#[cfg(kani)] #[kani::proof] fn error_source_none(){let e=arbitrary_error();assert!(<base16ct::Error as core::error::Error>::source(&e).is_none());}
#[cfg(kani)] #[kani::proof] fn error_cause_none(){let e=arbitrary_error();assert!(<base16ct::Error as core::error::Error>::cause(&e).is_none());}
#[cfg(kani)] #[kani::proof] #[kani::unwind(80)] fn error_description_literal(){let e=arbitrary_error();assert_eq!(<base16ct::Error as core::error::Error>::description(&e),"description() is deprecated; use Display");}
