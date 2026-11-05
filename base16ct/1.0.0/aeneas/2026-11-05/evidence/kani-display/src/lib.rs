#[cfg(kani)]
mod checks {
    use core::fmt::{self,Write};
    struct Sink { bytes:[u8;64], len:usize, fail:bool, nonempty_attempt_len:usize }
    impl Write for Sink {
        fn write_str(&mut self, text:&str)->fmt::Result {
            if !text.is_empty() {self.nonempty_attempt_len=text.len();}
            if self.fail { return Err(fmt::Error); }
            let end=self.len+text.len();
            if end>self.bytes.len() { return Err(fmt::Error); }
            self.bytes[self.len..end].copy_from_slice(text.as_bytes());
            self.len=end;
            Ok(())
        }
    }
    #[kani::proof]
    #[kani::unwind(80)]
    fn default_display_literals_and_writer_error() {
        let encoding:bool=kani::any();
        let fail:bool=kani::any();
        let error=if encoding {base16ct::Error::InvalidEncoding} else {base16ct::Error::InvalidLength};
        let expected_len:usize=if encoding {23} else {21};
        let expected_bytes:[u8;64]=if encoding {[105,110,118,97,108,105,100,32,66,97,115,101,49,54,32,101,110,99,111,100,105,110,103,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]} else {[105,110,118,97,108,105,100,32,66,97,115,101,49,54,32,108,101,110,103,116,104,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]};
        let mut sink=Sink {bytes:[0;64],len:0,fail,nonempty_attempt_len:0};
        let result=write!(&mut sink,"{}",error);
        // Witness that a nonempty literal reaches the writer even in Err mode.
        assert_eq!(sink.nonempty_attempt_len,expected_len);
        if fail {
            assert!(result.is_err());
            assert_eq!(sink.len,0);
        } else {
            assert!(result.is_ok());
            assert_eq!(sink.len,expected_len);
        }
        // Full fixed-capacity contents: literal prefix plus zero unused bytes,
        // or every byte unchanged at zero for the selected writer error.
        for i in 0..64 {
            let expected=if fail {0} else {expected_bytes[i]};
            assert_eq!(sink.bytes[i],expected);
        }
    }
}
