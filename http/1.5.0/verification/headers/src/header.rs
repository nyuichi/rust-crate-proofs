pub const MAX_HEADER_NAME_LEN: usize = (1 << 16) - 1;

#[cfg(not(http_header_value_leaf))]
#[path = "../../../src/header/name.rs"]
pub mod name;

#[cfg(not(http_header_name_leaf))]
#[path = "../../../src/header/value.rs"]
pub mod value;
