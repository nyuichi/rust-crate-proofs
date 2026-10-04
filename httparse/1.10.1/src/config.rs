//! Runtime parser configuration and its exact builder API model.

extern crate creusot_std;
#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite, View};

/// Parser configuration.
// The runtime Debug implementation is unchanged; its trait refinement is not
// part of the current Creusot target because the formatter spec remains open.
#[cfg_attr(not(creusot), derive(Clone, Debug))]
#[cfg_attr(creusot, derive(Clone))]
pub struct ParserConfig {
    pub(crate) allow_spaces_after_header_name_in_responses: bool,
    pub(crate) allow_obsolete_multiline_headers_in_responses: bool,
    pub(crate) allow_multiple_spaces_in_request_line_delimiters: bool,
    pub(crate) allow_multiple_spaces_in_response_status_delimiters: bool,
    pub(crate) allow_space_before_first_header_name: bool,
    pub(crate) ignore_invalid_headers_in_responses: bool,
    pub(crate) ignore_invalid_headers_in_requests: bool,
}

#[cfg(creusot)]
impl View for ParserConfig {
    type ViewTy = (bool, bool, bool, bool, bool, bool, bool);

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! {
            (
                self.allow_spaces_after_header_name_in_responses,
                self.allow_obsolete_multiline_headers_in_responses,
                self.allow_multiple_spaces_in_request_line_delimiters,
                self.allow_multiple_spaces_in_response_status_delimiters,
                self.allow_space_before_first_header_name,
                self.ignore_invalid_headers_in_responses,
                self.ignore_invalid_headers_in_requests,
            )
        }
    }
}

impl Default for ParserConfig {
    #[ensures(result@ == (false, false, false, false, false, false, false))]
    fn default() -> Self {
        Self {
            allow_spaces_after_header_name_in_responses: false,
            allow_obsolete_multiline_headers_in_responses: false,
            allow_multiple_spaces_in_request_line_delimiters: false,
            allow_multiple_spaces_in_response_status_delimiters: false,
            allow_space_before_first_header_name: false,
            ignore_invalid_headers_in_responses: false,
            ignore_invalid_headers_in_requests: false,
        }
    }
}

impl ParserConfig {
    /// Sets whether spaces and tabs should be allowed after header names in responses.
    #[ensures(result@ == (
        value,
        self@.1,
        self@.2,
        self@.3,
        self@.4,
        self@.5,
        self@.6,
    ))]
    #[ensures((^result)@ == (^self)@)]
    pub fn allow_spaces_after_header_name_in_responses(&mut self, value: bool) -> &mut Self {
        self.allow_spaces_after_header_name_in_responses = value;
        self
    }

    /// Sets whether multiple spaces are allowed as delimiters in request lines.
    ///
    /// # Background
    ///
    /// The [latest version of the HTTP/1.1 spec][spec] allows implementations to parse multiple
    /// whitespace characters in place of the `SP` delimiters in the request line, including:
    ///
    /// > SP, HTAB, VT (%x0B), FF (%x0C), or bare CR
    ///
    /// This option relaxes the parser to allow for multiple spaces, but does *not* allow the
    /// request line to contain the other mentioned whitespace characters.
    ///
    /// [spec]: https://httpwg.org/http-core/draft-ietf-httpbis-messaging-latest.html#rfc.section.3.p.3
    #[ensures(result@ == (
        self@.0,
        self@.1,
        value,
        self@.3,
        self@.4,
        self@.5,
        self@.6,
    ))]
    #[ensures((^result)@ == (^self)@)]
    pub fn allow_multiple_spaces_in_request_line_delimiters(&mut self, value: bool) -> &mut Self {
        self.allow_multiple_spaces_in_request_line_delimiters = value;
        self
    }

    /// Whether multiple spaces are allowed as delimiters in request lines.
    #[ensures(result == self@.2)]
    pub fn multiple_spaces_in_request_line_delimiters_are_allowed(&self) -> bool {
        self.allow_multiple_spaces_in_request_line_delimiters
    }

    /// Sets whether multiple spaces are allowed as delimiters in response status lines.
    ///
    /// # Background
    ///
    /// The [latest version of the HTTP/1.1 spec][spec] allows implementations to parse multiple
    /// whitespace characters in place of the `SP` delimiters in the response status line,
    /// including:
    ///
    /// > SP, HTAB, VT (%x0B), FF (%x0C), or bare CR
    ///
    /// This option relaxes the parser to allow for multiple spaces, but does *not* allow the status
    /// line to contain the other mentioned whitespace characters.
    ///
    /// [spec]: https://httpwg.org/http-core/draft-ietf-httpbis-messaging-latest.html#rfc.section.4.p.3
    #[ensures(result@ == (
        self@.0,
        self@.1,
        self@.2,
        value,
        self@.4,
        self@.5,
        self@.6,
    ))]
    #[ensures((^result)@ == (^self)@)]
    pub fn allow_multiple_spaces_in_response_status_delimiters(
        &mut self,
        value: bool,
    ) -> &mut Self {
        self.allow_multiple_spaces_in_response_status_delimiters = value;
        self
    }

    /// Whether multiple spaces are allowed as delimiters in response status lines.
    #[ensures(result == self@.3)]
    pub fn multiple_spaces_in_response_status_delimiters_are_allowed(&self) -> bool {
        self.allow_multiple_spaces_in_response_status_delimiters
    }

    /// Sets whether obsolete multiline headers should be allowed.
    ///
    /// This is an obsolete part of HTTP/1. Use at your own risk. If you are
    /// building an HTTP library, the newlines (`\r` and `\n`) should be
    /// replaced by spaces before handing the header value to the user.
    ///
    /// # Example
    ///
    /// ```rust
    /// let buf = b"HTTP/1.1 200 OK\r\nFolded-Header: hello\r\n there \r\n\r\n";
    /// let mut headers = [httparse::EMPTY_HEADER; 16];
    /// let mut response = httparse::Response::new(&mut headers);
    ///
    /// let res = httparse::ParserConfig::default()
    ///     .allow_obsolete_multiline_headers_in_responses(true)
    ///     .parse_response(&mut response, buf);
    ///
    /// assert_eq!(res, Ok(httparse::Status::Complete(buf.len())));
    ///
    /// assert_eq!(response.headers.len(), 1);
    /// assert_eq!(response.headers[0].name, "Folded-Header");
    /// assert_eq!(response.headers[0].value, b"hello\r\n there");
    /// ```
    #[ensures(result@ == (
        self@.0,
        value,
        self@.2,
        self@.3,
        self@.4,
        self@.5,
        self@.6,
    ))]
    #[ensures((^result)@ == (^self)@)]
    pub fn allow_obsolete_multiline_headers_in_responses(&mut self, value: bool) -> &mut Self {
        self.allow_obsolete_multiline_headers_in_responses = value;
        self
    }

    /// Whether obsolete multiline headers should be allowed.
    #[ensures(result == self@.1)]
    pub fn obsolete_multiline_headers_in_responses_are_allowed(&self) -> bool {
        self.allow_obsolete_multiline_headers_in_responses
    }

    /// Sets whether white space before the first header is allowed
    ///
    /// This is not allowed by spec but some browsers ignore it. So this an option for
    /// compatibility.
    /// See https://github.com/curl/curl/issues/11605 for reference
    /// # Example
    ///
    /// ```rust
    /// let buf = b"HTTP/1.1 200 OK\r\n Space-Before-Header: hello there\r\n\r\n";
    /// let mut headers = [httparse::EMPTY_HEADER; 1];
    /// let mut response = httparse::Response::new(&mut headers[..]);
    /// let result = httparse::ParserConfig::default()
    ///     .allow_space_before_first_header_name(true)
    ///     .parse_response(&mut response, buf);
    ///
    /// assert_eq!(result, Ok(httparse::Status::Complete(buf.len())));
    /// assert_eq!(response.version.unwrap(), 1);
    /// assert_eq!(response.code.unwrap(), 200);
    /// assert_eq!(response.reason.unwrap(), "OK");
    /// assert_eq!(response.headers.len(), 1);
    /// assert_eq!(response.headers[0].name, "Space-Before-Header");
    /// assert_eq!(response.headers[0].value, &b"hello there"[..]);
    /// ```
    #[ensures(result@ == (
        self@.0,
        self@.1,
        self@.2,
        self@.3,
        value,
        self@.5,
        self@.6,
    ))]
    #[ensures((^result)@ == (^self)@)]
    pub fn allow_space_before_first_header_name(&mut self, value: bool) -> &mut Self {
        self.allow_space_before_first_header_name = value;
        self
    }

    /// Whether white space before first header is allowed or not
    #[ensures(result == self@.4)]
    pub fn space_before_first_header_name_are_allowed(&self) -> bool {
        self.allow_space_before_first_header_name
    }

    /// Sets whether invalid header lines should be silently ignored in responses.
    ///
    /// This mimicks the behaviour of major browsers. You probably don't want this.
    /// You should only want this if you are implementing a proxy whose main
    /// purpose is to sit in front of browsers whose users access arbitrary content
    /// which may be malformed, and they expect everything that works without
    /// the proxy to keep working with the proxy.
    ///
    /// This option will prevent `ParserConfig::parse_response` from returning
    /// an error encountered when parsing a header, except if the error was caused
    /// by the character NUL (ASCII code 0), as Chrome specifically always reject
    /// those, or if the error was caused by a lone character `\r`, as Firefox and
    /// Chrome behave differently in that case.
    ///
    /// The ignorable errors are:
    /// * empty header names;
    /// * characters that are not allowed in header names, except for `\0` and `\r`;
    /// * when `allow_spaces_after_header_name_in_responses` is not enabled,
    ///   spaces and tabs between the header name and the colon;
    /// * missing colon between header name and value;
    /// * when `allow_obsolete_multiline_headers_in_responses` is not enabled,
    ///   headers using obsolete line folding.
    /// * characters that are not allowed in header values except for `\0` and `\r`.
    ///
    /// If an ignorable error is encountered, the parser tries to find the next
    /// line in the input to resume parsing the rest of the headers. As lines
    /// contributing to a header using obsolete line folding always start
    /// with whitespace, those will be ignored too. An error will be emitted
    /// nonetheless if it finds `\0` or a lone `\r` while looking for the
    /// next line.
    #[ensures(result@ == (
        self@.0,
        self@.1,
        self@.2,
        self@.3,
        self@.4,
        value,
        self@.6,
    ))]
    #[ensures((^result)@ == (^self)@)]
    pub fn ignore_invalid_headers_in_responses(&mut self, value: bool) -> &mut Self {
        self.ignore_invalid_headers_in_responses = value;
        self
    }

    /// Sets whether invalid header lines should be silently ignored in requests.
    #[ensures(result@ == (
        self@.0,
        self@.1,
        self@.2,
        self@.3,
        self@.4,
        self@.5,
        value,
    ))]
    #[ensures((^result)@ == (^self)@)]
    pub fn ignore_invalid_headers_in_requests(&mut self, value: bool) -> &mut Self {
        self.ignore_invalid_headers_in_requests = value;
        self
    }
}
