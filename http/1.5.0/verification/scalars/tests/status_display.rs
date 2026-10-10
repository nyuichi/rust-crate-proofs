use std::fmt::{self, Write};

use http_scalar_proofs::status::StatusCode;

struct OriginalStatusDisplay(StatusCode);

impl fmt::Display for OriginalStatusDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            u16::from(self.0),
            self.0.canonical_reason().unwrap_or("<unknown status code>")
        )
    }
}

#[derive(Default)]
struct RecordingWriter {
    writes: Vec<String>,
    fail_on_write: Option<usize>,
}

impl Write for RecordingWriter {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.writes.push(text.to_owned());
        if self.fail_on_write == Some(self.writes.len()) {
            Err(fmt::Error)
        } else {
            Ok(())
        }
    }
}

fn render_with_failure<T: fmt::Display>(
    value: &T,
    fail_on_write: Option<usize>,
) -> (fmt::Result, Vec<String>) {
    let mut writer = RecordingWriter {
        fail_on_write,
        ..RecordingWriter::default()
    };
    let result = fmt::write(&mut writer, format_args!("{}", value));
    (result, writer.writes)
}

#[test]
fn display_keeps_output_flags_and_partial_write_behavior() {
    for code in [200, 299] {
        let status = StatusCode::from_u16(code).unwrap();
        let original = OriginalStatusDisplay(status);

        assert_eq!(format!("{}", status), format!("{}", original));
        assert_eq!(format!("{:>12.2}", status), format!("{:>12.2}", original));
        assert_eq!(format!("{:+010}", status), format!("{:+010}", original));

        for failing_write in [Some(1), Some(2), Some(3), None] {
            assert_eq!(
                render_with_failure(&status, failing_write),
                render_with_failure(&original, failing_write),
                "status {code}, failure at {failing_write:?}"
            );
        }
    }
}
