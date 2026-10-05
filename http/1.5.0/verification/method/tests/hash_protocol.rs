use std::hash::{Hash, Hasher};

use http_method_proofs::method::Method;

#[derive(Hash)]
struct DerivedInline([u8; 15], u8);

#[derive(Hash)]
struct DerivedAllocated(Box<[u8]>);

// This mirrors the original Inner declaration and derive(Hash) expansion.
#[derive(Hash)]
enum DerivedInner {
    Options,
    Get,
    Post,
    Put,
    Delete,
    Head,
    Trace,
    Connect,
    Patch,
    Query,
    ExtensionInline(DerivedInline),
    ExtensionAllocated(DerivedAllocated),
}

#[derive(Debug, PartialEq, Eq)]
enum Event {
    Write(Vec<u8>),
    U8(u8),
    U16(u16),
    Usize(usize),
    Isize(isize),
    LengthPrefix(usize),
}

#[derive(Default)]
struct RecordingHasher(Vec<Event>);

impl Hasher for RecordingHasher {
    fn finish(&self) -> u64 {
        0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.push(Event::Write(bytes.to_vec()));
    }

    fn write_u8(&mut self, value: u8) {
        self.0.push(Event::U8(value));
    }

    fn write_u16(&mut self, value: u16) {
        self.0.push(Event::U16(value));
    }

    fn write_usize(&mut self, value: usize) {
        self.0.push(Event::Usize(value));
    }

    fn write_isize(&mut self, value: isize) {
        self.0.push(Event::Isize(value));
    }

    fn write_length_prefix(&mut self, len: usize) {
        self.0.push(Event::LengthPrefix(len));
    }
}

fn trace<T: Hash>(value: &T) -> Vec<Event> {
    let mut hasher = RecordingHasher::default();
    value.hash(&mut hasher);
    hasher.0
}

fn inline_model(bytes: &[u8]) -> DerivedInner {
    assert!(bytes.len() <= 15);
    let mut data = [0; 15];
    data[..bytes.len()].copy_from_slice(bytes);
    DerivedInner::ExtensionInline(DerivedInline(data, bytes.len() as u8))
}

#[test]
fn method_hash_keeps_the_original_derived_callback_protocol() {
    let builtins = [
        (Method::OPTIONS, DerivedInner::Options),
        (Method::GET, DerivedInner::Get),
        (Method::POST, DerivedInner::Post),
        (Method::PUT, DerivedInner::Put),
        (Method::DELETE, DerivedInner::Delete),
        (Method::HEAD, DerivedInner::Head),
        (Method::TRACE, DerivedInner::Trace),
        (Method::CONNECT, DerivedInner::Connect),
        (Method::PATCH, DerivedInner::Patch),
        (Method::QUERY, DerivedInner::Query),
    ];

    for (actual, derived) in builtins {
        assert_eq!(trace(&actual), trace(&derived));
    }

    let inline_bytes = b"CUSTOM";
    let inline = Method::from_bytes(inline_bytes).unwrap();
    assert_eq!(trace(&inline), trace(&inline_model(inline_bytes)));

    let allocated_bytes = b"EXTENSION-METHOD-OVER-15";
    let allocated = Method::from_bytes(allocated_bytes).unwrap();
    let derived_allocated = DerivedInner::ExtensionAllocated(DerivedAllocated(
        allocated_bytes.to_vec().into_boxed_slice(),
    ));
    assert_eq!(trace(&allocated), trace(&derived_allocated));
}
