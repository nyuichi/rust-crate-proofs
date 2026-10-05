use std::hash::{Hash, Hasher};

use http_scalar_proofs::version::Version;

#[derive(Hash)]
enum DerivedHttp {
    Http09,
    Http10,
    Http11,
    H2,
    H3,
    __NonExhaustive,
}

#[derive(Hash)]
struct DerivedVersion(DerivedHttp);

#[derive(Debug, PartialEq, Eq)]
enum Event {
    Write(Vec<u8>),
    Isize(isize),
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

    fn write_isize(&mut self, value: isize) {
        self.0.push(Event::Isize(value));
    }
}

fn trace<T: Hash>(value: &T) -> Vec<Event> {
    let mut hasher = RecordingHasher::default();
    value.hash(&mut hasher);
    hasher.0
}

#[test]
fn version_hash_keeps_the_original_derived_callback_protocol() {
    let cases = [
        (Version::HTTP_09, DerivedVersion(DerivedHttp::Http09)),
        (Version::HTTP_10, DerivedVersion(DerivedHttp::Http10)),
        (Version::HTTP_11, DerivedVersion(DerivedHttp::Http11)),
        (Version::HTTP_2, DerivedVersion(DerivedHttp::H2)),
        (Version::HTTP_3, DerivedVersion(DerivedHttp::H3)),
    ];

    for (actual, derived) in cases {
        assert_eq!(trace(&actual), trace(&derived));
    }
}
