#[cfg(kani)]
mod checks {
    use core::fmt::{self,Write};
    struct Sink { bytes:[u8;64], len:usize, fail:bool }
    impl Write for Sink {
        fn write_str(&mut self, text:&str)->fmt::Result {
            if self.fail { return Err(fmt::Error); }
            let end=self.len+text.len();
            if end>self.bytes.len() { return Err(fmt::Error); }
            self.bytes[self.len..end].copy_from_slice(text.as_bytes());
            self.len=end;
            Ok(())
        }
    }
    #[kani::proof]
    fn default_display_literals_and_writer_error() {
        let encoding:bool=kani::any();
        let fail:bool=kani::any();
        let error=if encoding {base16ct::Error::InvalidEncoding} else {base16ct::Error::InvalidLength};
        let expected:&[u8]=if encoding {b"invalid Base16 encoding"} else {b"invalid Base16 length"};
        let mut sink=Sink {bytes:[0;64],len:0,fail};
        let result=write!(&mut sink,"{}",error);
        if fail {
            assert!(result.is_err());
            assert_eq!(sink.len,0);
        } else {
            assert!(result.is_ok());
            assert_eq!(sink.len,expected.len());
            assert_eq!(&sink.bytes[..sink.len],expected);
        }
    }
}
