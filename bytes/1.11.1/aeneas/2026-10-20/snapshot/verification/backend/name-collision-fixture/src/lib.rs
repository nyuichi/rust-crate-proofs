pub mod bytes {
    pub struct Bytes {
        value: u8,
        vtable: u8,
    }

    static STATIC_VTABLE: u8 = 7;

    impl Bytes {
        pub fn from_static(bytes: u8) -> Self {
            let vtable = STATIC_VTABLE;
            Self { value: bytes, vtable }
        }
    }
}
