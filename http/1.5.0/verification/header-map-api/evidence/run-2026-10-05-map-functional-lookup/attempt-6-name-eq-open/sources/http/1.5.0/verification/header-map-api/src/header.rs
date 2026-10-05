pub const MAX_HEADER_NAME_LEN: usize = (1 << 16) - 1;

#[path = "../../../src/header/name.rs"]
pub mod name;
pub use http_runtime::header::HeaderValue;
#[path = "../../../src/header/map_capacity.rs"]
mod map_capacity;
#[path = "../../../src/header/map.rs"]
pub mod map;

pub use map::HeaderMap;
pub use name::{HeaderName, InvalidHeaderName};
