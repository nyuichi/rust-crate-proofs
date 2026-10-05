//! HTTP response types.
//!
//! This module contains structs related to HTTP responses, notably the
//! `Response` type itself as well as a builder to create responses. Typically
//! you'll import the `http::Response` type rather than reaching into this
//! module itself.
//!
//! # Examples
//!
//! Creating a `Response` to return
//!
//! ```
//! use http::{Request, Response, StatusCode};
//!
//! fn respond_to(req: Request<()>) -> http::Result<Response<()>> {
//!     let mut builder = Response::builder()
//!         .header("Foo", "Bar")
//!         .status(StatusCode::OK);
//!
//!     if req.headers().contains_key("Another-Header") {
//!         builder = builder.header("Another-Header", "Ack");
//!     }
//!
//!     builder.body(())
//! }
//! ```
//!
//! A simple 404 handler
//!
//! ```
//! use http::{Request, Response, StatusCode};
//!
//! fn not_found(_req: Request<()>) -> http::Result<Response<()>> {
//!     Response::builder()
//!         .status(StatusCode::NOT_FOUND)
//!         .body(())
//! }
//! ```
//!
//! Or otherwise inspecting the result of a request:
//!
//! ```no_run
//! use http::{Request, Response};
//!
//! fn get(url: &str) -> http::Result<Response<()>> {
//!     // ...
//! # panic!()
//! }
//!
//! let response = get("https://www.rust-lang.org/").unwrap();
//!
//! if !response.status().is_success() {
//!     panic!("failed to get a successful response status!");
//! }
//!
//! if let Some(date) = response.headers().get("Date") {
//!     // we've got a `Date` header!
//! }
//!
//! let body = response.body();
//! // ...
//! ```

use std::any::Any;
use std::convert::TryInto;
use std::fmt;

#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite, requires, DeepModel};

use crate::header::{HeaderMap, HeaderName, HeaderValue};
use crate::status::StatusCode;
use crate::version::Version;
use crate::{Extensions, Result};

#[cfg(creusot)]
use creusot_std::std::ops::{FnExt as _, FnOnceExt as _};

/// Represents an HTTP response
///
/// An HTTP response consists of a head and a potentially optional body. The body
/// component is generic, enabling arbitrary types to represent the HTTP body.
/// For example, the body could be `Vec<u8>`, a `Stream` of byte chunks, or a
/// value that has been deserialized.
///
/// Typically you'll work with responses on the client side as the result of
/// sending a `Request` and on the server you'll be generating a `Response` to
/// send back to the client.
///
/// # Examples
///
/// Creating a `Response` to return
///
/// ```
/// use http::{Request, Response, StatusCode};
///
/// fn respond_to(req: Request<()>) -> http::Result<Response<()>> {
///     let mut builder = Response::builder()
///         .header("Foo", "Bar")
///         .status(StatusCode::OK);
///
///     if req.headers().contains_key("Another-Header") {
///         builder = builder.header("Another-Header", "Ack");
///     }
///
///     builder.body(())
/// }
/// ```
///
/// A simple 404 handler
///
/// ```
/// use http::{Request, Response, StatusCode};
///
/// fn not_found(_req: Request<()>) -> http::Result<Response<()>> {
///     Response::builder()
///         .status(StatusCode::NOT_FOUND)
///         .body(())
/// }
/// ```
///
/// Or otherwise inspecting the result of a request:
///
/// ```no_run
/// use http::{Request, Response};
///
/// fn get(url: &str) -> http::Result<Response<()>> {
///     // ...
/// # panic!()
/// }
///
/// let response = get("https://www.rust-lang.org/").unwrap();
///
/// if !response.status().is_success() {
///     panic!("failed to get a successful response status!");
/// }
///
/// if let Some(date) = response.headers().get("Date") {
///     // we've got a `Date` header!
/// }
///
/// let body = response.body();
/// // ...
/// ```
///
/// Deserialize a response of bytes via json:
///
/// ```
/// use http::Response;
/// use serde::de;
///
/// fn deserialize<T>(res: Response<Vec<u8>>) -> serde_json::Result<Response<T>>
///     where for<'de> T: de::Deserialize<'de>,
/// {
///     let (parts, body) = res.into_parts();
///     let body = serde_json::from_slice(&body)?;
///     Ok(Response::from_parts(parts, body))
/// }
/// #
/// # fn main() {}
/// ```
///
/// Or alternatively, serialize the body of a response to json
///
/// ```
/// use http::Response;
/// use serde::ser;
///
/// fn serialize<T>(res: Response<T>) -> serde_json::Result<Response<Vec<u8>>>
///     where T: ser::Serialize,
/// {
///     let (parts, body) = res.into_parts();
///     let body = serde_json::to_vec(&body)?;
///     Ok(Response::from_parts(parts, body))
/// }
/// #
/// # fn main() {}
/// ```
#[allow(unexpected_cfgs)]
#[cfg_attr(not(any(http_composition_leaf, http_builder_entrypoints_leaf)), derive(Clone))]
pub struct Response<T> {
    head: Parts,
    body: T,
}

/// Component parts of an HTTP `Response`
///
/// The HTTP response head consists of a status, version, and a set of
/// header fields.
#[allow(unexpected_cfgs)]
#[cfg_attr(not(any(http_composition_leaf, http_builder_entrypoints_leaf)), derive(Clone))]
pub struct Parts {
    /// The response's status
    pub status: StatusCode,

    /// The response's version
    pub version: Version,

    /// The response's headers
    pub headers: HeaderMap<HeaderValue>,

    /// The response's extensions
    pub extensions: Extensions,

    _priv: (),
}

/// An HTTP response builder
///
/// This type can be used to construct an instance of `Response` through a
/// builder-like pattern.
#[cfg_attr(not(any(http_composition_leaf, http_builder_entrypoints_leaf)), derive(Debug))]
pub struct Builder {
    inner: Result<Parts>,
}

/// A specification-only projection of a response's head.
#[cfg(creusot)]
#[logic]
pub fn response_head<T>(response: Response<T>) -> Parts {
    response.head
}

/// A specification-only projection of a response's body.
#[cfg(creusot)]
#[logic]
pub fn response_body<T>(response: Response<T>) -> T {
    response.body
}

/// Whether a response builder currently contains a valid head.
#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn response_builder_is_valid(builder: &Builder) -> bool {
    match &builder.inner {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// The status and version fields initialized by the default builder.
#[cfg(all(creusot, any(not(http_composition_leaf), http_composition_uri_defaults_leaf)))]
#[doc(hidden)]
#[logic]
pub fn response_builder_has_default_core_fields(builder: &Builder) -> bool {
    pearlite! {
        match response_builder_parts(builder) {
            Some(parts) => parts.status.deep_model() == 200 && parts.version.deep_model() == 11,
            None => false,
        }
    }
}

/// The exact stored error, including its concrete payload, when a builder is
/// in the error state.
#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn response_builder_error<'a>(builder: &'a Builder) -> Option<crate::ErrorModelRef<'a>> {
    match &builder.inner {
        Ok(_) => None,
        Err(error) => Some(crate::error_model_ref(error)),
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn response_builder_error_is_from<E>(builder: &Builder, source: E) -> bool
where
    E: Into<crate::Error>,
{
    match &builder.inner {
        Ok(_) => false,
        Err(error) => <E as Into<crate::Error>>::into.postcondition((source,), *error),
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn response_builder_status<'a>(builder: &'a Builder) -> Option<&'a StatusCode> {
    match &builder.inner {
        Ok(head) => Some(&head.status),
        Err(_) => None,
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn response_builder_version<'a>(builder: &'a Builder) -> Option<&'a Version> {
    match &builder.inner {
        Ok(head) => Some(&head.version),
        Err(_) => None,
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn response_builder_headers<'a>(builder: &'a Builder) -> Option<&'a HeaderMap<HeaderValue>> {
    match &builder.inner {
        Ok(head) => Some(&head.headers),
        Err(_) => None,
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn response_builder_extensions<'a>(builder: &'a Builder) -> Option<&'a Extensions> {
    match &builder.inner {
        Ok(head) => Some(&head.extensions),
        Err(_) => None,
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn response_builder_parts<'a>(builder: &'a Builder) -> Option<&'a Parts> {
    match &builder.inner {
        Ok(head) => Some(head),
        Err(_) => None,
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn response_result_head<'a, T>(outcome: &'a Result<Response<T>>) -> Option<&'a Parts> {
    match outcome {
        Ok(response) => Some(&response.head),
        Err(_) => None,
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn response_result_body<'a, T>(outcome: &'a Result<Response<T>>) -> Option<&'a T> {
    match outcome {
        Ok(response) => Some(&response.body),
        Err(_) => None,
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn response_result_error<'a, T>(outcome: &'a Result<Response<T>>) -> Option<crate::ErrorModelRef<'a>> {
    match outcome {
        Ok(_) => None,
        Err(error) => Some(crate::error_model_ref(error)),
    }
}

#[allow(unexpected_cfgs)]
#[cfg(any(not(http_composition_leaf), http_builder_entrypoints_leaf))]
impl Response<()> {
    /// Creates a new builder-style object to manufacture a `Response`
    ///
    /// This method returns an instance of `Builder` which can be used to
    /// create a `Response`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let response = Response::builder()
    ///     .status(200)
    ///     .header("X-Custom-Foo", "Bar")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[inline]
    #[cfg_attr(creusot, ensures(response_builder_is_valid(&result)))]
    #[cfg_attr(all(creusot, any(not(http_composition_leaf), http_composition_uri_defaults_leaf)), ensures(
        response_builder_has_default_core_fields(&result)
    ))]
    pub fn builder() -> Builder {
        Builder::new()
    }
}

impl<T> Response<T> {
    /// Creates a new blank `Response` with the body
    ///
    /// The component parts of this response will be set to their default, e.g.
    /// the ok status, no headers, etc.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let response = Response::new("hello world");
    ///
    /// assert_eq!(response.status(), StatusCode::OK);
    /// assert_eq!(*response.body(), "hello world");
    /// ```
    #[inline]
    pub fn new(body: T) -> Response<T> {
        Response {
            head: Parts::new(),
            body,
        }
    }

    /// Creates a new `Response` with the given head and body
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let response = Response::new("hello world");
    /// let (mut parts, body) = response.into_parts();
    ///
    /// parts.status = StatusCode::BAD_REQUEST;
    /// let response = Response::from_parts(parts, body);
    ///
    /// assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    /// assert_eq!(*response.body(), "hello world");
    /// ```
    #[inline]
    #[ensures(response_head(result) == parts && response_body(result) == body)]
    pub fn from_parts(parts: Parts, body: T) -> Response<T> {
        Response { head: parts, body }
    }

    /// Returns the `StatusCode`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let response: Response<()> = Response::default();
    /// assert_eq!(response.status(), StatusCode::OK);
    /// ```
    #[inline]
    #[ensures(result == response_head(*self).status)]
    pub fn status(&self) -> StatusCode {
        self.head.status
    }

    /// Returns a mutable reference to the associated `StatusCode`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let mut response: Response<()> = Response::default();
    /// *response.status_mut() = StatusCode::CREATED;
    /// assert_eq!(response.status(), StatusCode::CREATED);
    /// ```
    #[inline]
    #[ensures(*result == response_head(*self).status)]
    #[ensures(^result == response_head(^self).status)]
    #[ensures(response_head(^self).version == response_head(*self).version)]
    #[ensures(response_head(^self).headers == response_head(*self).headers)]
    #[ensures(response_head(^self).extensions == response_head(*self).extensions)]
    #[ensures(response_body(^self) == response_body(*self))]
    pub fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.head.status
    }

    /// Returns a reference to the associated version.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let response: Response<()> = Response::default();
    /// assert_eq!(response.version(), Version::HTTP_11);
    /// ```
    #[inline]
    #[ensures(result == response_head(*self).version)]
    pub fn version(&self) -> Version {
        self.head.version
    }

    /// Returns a mutable reference to the associated version.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let mut response: Response<()> = Response::default();
    /// *response.version_mut() = Version::HTTP_2;
    /// assert_eq!(response.version(), Version::HTTP_2);
    /// ```
    #[inline]
    #[ensures(*result == response_head(*self).version)]
    #[ensures(^result == response_head(^self).version)]
    #[ensures(response_head(^self).status == response_head(*self).status)]
    #[ensures(response_head(^self).headers == response_head(*self).headers)]
    #[ensures(response_head(^self).extensions == response_head(*self).extensions)]
    #[ensures(response_body(^self) == response_body(*self))]
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.head.version
    }

    /// Returns a reference to the associated header field map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let response: Response<()> = Response::default();
    /// assert!(response.headers().is_empty());
    /// ```
    #[inline]
    #[ensures(*result == response_head(*self).headers)]
    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        &self.head.headers
    }

    /// Returns a mutable reference to the associated header field map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// # use http::header::*;
    /// let mut response: Response<()> = Response::default();
    /// response.headers_mut().insert(HOST, HeaderValue::from_static("world"));
    /// assert!(!response.headers().is_empty());
    /// ```
    #[inline]
    #[ensures(*result == response_head(*self).headers)]
    #[ensures(^result == response_head(^self).headers)]
    #[ensures(response_head(^self).status == response_head(*self).status)]
    #[ensures(response_head(^self).version == response_head(*self).version)]
    #[ensures(response_head(^self).extensions == response_head(*self).extensions)]
    #[ensures(response_body(^self) == response_body(*self))]
    pub fn headers_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
        &mut self.head.headers
    }

    /// Returns a reference to the associated extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let response: Response<()> = Response::default();
    /// assert!(response.extensions().get::<i32>().is_none());
    /// ```
    #[inline]
    #[ensures(*result == response_head(*self).extensions)]
    pub fn extensions(&self) -> &Extensions {
        &self.head.extensions
    }

    /// Returns a mutable reference to the associated extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// # use http::header::*;
    /// let mut response: Response<()> = Response::default();
    /// response.extensions_mut().insert("hello");
    /// assert_eq!(response.extensions().get(), Some(&"hello"));
    /// ```
    #[inline]
    #[ensures(*result == response_head(*self).extensions)]
    #[ensures(^result == response_head(^self).extensions)]
    #[ensures(response_head(^self).status == response_head(*self).status)]
    #[ensures(response_head(^self).version == response_head(*self).version)]
    #[ensures(response_head(^self).headers == response_head(*self).headers)]
    #[ensures(response_body(^self) == response_body(*self))]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.head.extensions
    }

    /// Returns a reference to the associated HTTP body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let response: Response<String> = Response::default();
    /// assert!(response.body().is_empty());
    /// ```
    #[inline]
    #[ensures(*result == response_body(*self))]
    pub fn body(&self) -> &T {
        &self.body
    }

    /// Returns a mutable reference to the associated HTTP body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let mut response: Response<String> = Response::default();
    /// response.body_mut().push_str("hello world");
    /// assert!(!response.body().is_empty());
    /// ```
    #[inline]
    #[ensures(*result == response_body(*self))]
    #[ensures(^result == response_body(^self))]
    #[ensures(response_head(^self) == response_head(*self))]
    pub fn body_mut(&mut self) -> &mut T {
        &mut self.body
    }

    /// Consumes the response, returning just the body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::Response;
    /// let response = Response::new(10);
    /// let body = response.into_body();
    /// assert_eq!(body, 10);
    /// ```
    #[inline]
    #[ensures(result == response_body(self))]
    pub fn into_body(self) -> T {
        self.body
    }

    /// Consumes the response returning the head and body parts.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let response: Response<()> = Response::default();
    /// let (parts, body) = response.into_parts();
    /// assert_eq!(parts.status, StatusCode::OK);
    /// ```
    #[inline]
    #[ensures(result.0 == response_head(self) && result.1 == response_body(self))]
    pub fn into_parts(self) -> (Parts, T) {
        (self.head, self.body)
    }

    /// Consumes the response returning a new response with body mapped to the
    /// return type of the passed in function.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let response = Response::builder().body("some string").unwrap();
    /// let mapped_response: Response<&[u8]> = response.map(|b| {
    ///   assert_eq!(b, "some string");
    ///   b.as_bytes()
    /// });
    /// assert_eq!(mapped_response.body(), &"some string".as_bytes());
    /// ```
    #[inline]
    #[requires(f.precondition((response_body(self),)))]
    #[ensures(response_head(result) == response_head(self))]
    #[ensures(f.postcondition_once((response_body(self),), response_body(result)))]
    pub fn map<F, U>(self, f: F) -> Response<U>
    where
        F: FnOnce(T) -> U,
    {
        Response {
            body: f(self.body),
            head: self.head,
        }
    }
}

impl<T: Default> Default for Response<T> {
    #[inline]
    fn default() -> Response<T> {
        Response::new(T::default())
    }
}

impl<T: fmt::Debug> fmt::Debug for Response<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Response")
            .field("status", &self.status())
            .field("version", &self.version())
            .field("headers", self.headers())
            // omits Extensions because not useful
            .field("body", self.body())
            .finish()
    }
}

impl Parts {
    /// Creates a new default instance of `Parts`
    #[cfg_attr(all(creusot, any(not(http_composition_leaf), http_composition_uri_defaults_leaf)), ensures(
        result.status.deep_model() == 200
    ))]
    #[cfg_attr(all(creusot, any(not(http_composition_leaf), http_composition_uri_defaults_leaf)), ensures(
        result.version.deep_model() == 11
    ))]
    fn new() -> Parts {
        Parts {
            status: StatusCode::default(),
            version: Version::default(),
            headers: HeaderMap::default(),
            extensions: Extensions::default(),
            _priv: (),
        }
    }
}

impl fmt::Debug for Parts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parts")
            .field("status", &self.status)
            .field("version", &self.version)
            .field("headers", &self.headers)
            // omits Extensions because not useful
            // omits _priv because not useful
            .finish()
    }
}

impl Builder {
    /// Creates a new default instance of `Builder` to construct either a
    /// `Head` or a `Response`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let response = response::Builder::new()
    ///     .status(200)
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[inline]
    #[cfg_attr(creusot, ensures(response_builder_is_valid(&result)))]
    #[cfg_attr(all(creusot, any(not(http_composition_leaf), http_composition_uri_defaults_leaf)), ensures(
        response_builder_has_default_core_fields(&result)
    ))]
    pub fn new() -> Builder {
        Builder::default()
    }

    /// Set the HTTP status for this response.
    ///
    /// By default this is `200`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let response = Response::builder()
    ///     .status(200)
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, requires(!response_builder_is_valid(&self) || (
        <T as TryInto<StatusCode>>::try_into.precondition((status,))
        && forall<e: <T as TryInto<StatusCode>>::Error>
            <T as TryInto<StatusCode>>::try_into.postcondition((status,), Err(e))
                ==> <<T as TryInto<StatusCode>>::Error as Into<crate::Error>>::into.precondition((e,))
    )))]
    #[cfg_attr(creusot, ensures(!response_builder_is_valid(&self) ==> response_builder_error(&result) == response_builder_error(&self)))]
    #[cfg_attr(creusot, ensures(response_builder_is_valid(&self) ==> exists<converted: std::result::Result<StatusCode, <T as TryInto<StatusCode>>::Error>> (
        <T as TryInto<StatusCode>>::try_into.postcondition((status,), converted)
        && match converted {
            Ok(new_status) => response_builder_is_valid(&result)
                && response_builder_status(&result) == Some(&new_status)
                && response_builder_error(&result) == None
                && response_builder_version(&result) == response_builder_version(&self)
                && response_builder_headers(&result) == response_builder_headers(&self)
                && response_builder_extensions(&result) == response_builder_extensions(&self),
            Err(conversion_error) => !response_builder_is_valid(&result)
                && response_builder_error_is_from(&result, conversion_error)
                && response_builder_status(&result) == None
                && response_builder_version(&result) == None
                && response_builder_headers(&result) == None
                && response_builder_extensions(&result) == None,
        }
    )))]
    pub fn status<T>(self, status: T) -> Builder
    where
        T: TryInto<StatusCode>,
        <T as TryInto<StatusCode>>::Error: Into<crate::Error>,
    {
        self.and_then(move |mut head| {
            head.status = match status.try_into() {
                Ok(status) => status,
                Err(error) => return Err(error.into()),
            };
            Ok(head)
        })
    }

    /// Set the HTTP version for this response.
    ///
    /// By default this is HTTP/1.1
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let response = Response::builder()
    ///     .version(Version::HTTP_2)
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, ensures(response_builder_error(&result) == response_builder_error(&self)))]
    #[cfg_attr(creusot, ensures(response_builder_version(&result) == if response_builder_is_valid(&self) { Some(&version) } else { None }))]
    #[cfg_attr(creusot, ensures(response_builder_status(&result) == response_builder_status(&self)))]
    #[cfg_attr(creusot, ensures(response_builder_headers(&result) == response_builder_headers(&self)))]
    #[cfg_attr(creusot, ensures(response_builder_extensions(&result) == response_builder_extensions(&self)))]
    pub fn version(self, version: Version) -> Builder {
        let inner = match self.inner {
            Ok(mut head) => {
                head.version = version;
                Ok(head)
            }
            Err(error) => Err(error),
        };
        Builder { inner }
    }

    /// Appends a header to this response builder.
    ///
    /// This function will append the provided key/value as a header to the
    /// internal `HeaderMap` being constructed. Essentially this is equivalent
    /// to calling `HeaderMap::append`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// # use http::header::HeaderValue;
    ///
    /// let response = Response::builder()
    ///     .header("Content-Type", "text/html")
    ///     .header("X-Custom-Foo", "bar")
    ///     .header("content-length", 0)
    ///     .body(())
    ///     .unwrap();
    /// ```
    pub fn header<K, V>(self, key: K, value: V) -> Builder
    where
        K: TryInto<HeaderName>,
        <K as TryInto<HeaderName>>::Error: Into<crate::Error>,
        V: TryInto<HeaderValue>,
        <V as TryInto<HeaderValue>>::Error: Into<crate::Error>,
    {
        self.and_then(move |mut head| {
            let name = match key.try_into() {
                Ok(name) => name,
                Err(error) => return Err(error.into()),
            };
            let value = match value.try_into() {
                Ok(value) => value,
                Err(error) => return Err(error.into()),
            };
            head.headers.try_append(name, value)?;
            Ok(head)
        })
    }

    /// Get header on this response builder.
    ///
    /// When builder has error returns None.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::Response;
    /// # use http::header::HeaderValue;
    /// let res = Response::builder()
    ///     .header("Accept", "text/html")
    ///     .header("X-Custom-Foo", "bar");
    /// let headers = res.headers_ref().unwrap();
    /// assert_eq!( headers["Accept"], "text/html" );
    /// assert_eq!( headers["X-Custom-Foo"], "bar" );
    /// ```
    #[cfg_attr(creusot, ensures(result == response_builder_headers(self)))]
    pub fn headers_ref(&self) -> Option<&HeaderMap<HeaderValue>> {
        self.inner.as_ref().ok().map(|h| &h.headers)
    }

    /// Get header on this response builder.
    /// when builder has error returns None
    ///
    /// # Example
    ///
    /// ```
    /// # use http::*;
    /// # use http::header::HeaderValue;
    /// # use http::response::Builder;
    /// let mut res = Response::builder();
    /// {
    ///   let headers = res.headers_mut().unwrap();
    ///   headers.insert("Accept", HeaderValue::from_static("text/html"));
    ///   headers.insert("X-Custom-Foo", HeaderValue::from_static("bar"));
    /// }
    /// let headers = res.headers_ref().unwrap();
    /// assert_eq!( headers["Accept"], "text/html" );
    /// assert_eq!( headers["X-Custom-Foo"], "bar" );
    /// ```
    #[cfg_attr(creusot, ensures(match result {
        Some(reference) => Some(&*reference) == response_builder_headers(&*self),
        None => response_builder_headers(&*self) == None,
    }))]
    #[cfg_attr(creusot, ensures(match result {
        Some(reference) => Some(&^reference) == response_builder_headers(&^self),
        None => response_builder_headers(&^self) == None,
    }))]
    #[cfg_attr(creusot, ensures(response_builder_error(&^self) == response_builder_error(&*self)))]
    #[cfg_attr(creusot, ensures(response_builder_status(&^self) == response_builder_status(&*self)))]
    #[cfg_attr(creusot, ensures(response_builder_version(&^self) == response_builder_version(&*self)))]
    #[cfg_attr(creusot, ensures(response_builder_extensions(&^self) == response_builder_extensions(&*self)))]
    pub fn headers_mut(&mut self) -> Option<&mut HeaderMap<HeaderValue>> {
        match &mut self.inner {
            Ok(head) => Some(&mut head.headers),
            Err(_) => None,
        }
    }

    /// Adds an extension to this builder
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let response = Response::builder()
    ///     .extension("My Extension")
    ///     .body(())
    ///     .unwrap();
    ///
    /// assert_eq!(response.extensions().get::<&'static str>(),
    ///            Some(&"My Extension"));
    /// ```
    pub fn extension<T>(self, extension: T) -> Builder
    where
        T: Clone + Any + Send + Sync + 'static,
    {
        self.and_then(move |mut head| {
            head.extensions.insert(extension);
            Ok(head)
        })
    }

    /// Get a reference to the extensions for this response builder.
    ///
    /// If the builder has an error, this returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::Response;
    /// let res = Response::builder().extension("My Extension").extension(5u32);
    /// let extensions = res.extensions_ref().unwrap();
    /// assert_eq!(extensions.get::<&'static str>(), Some(&"My Extension"));
    /// assert_eq!(extensions.get::<u32>(), Some(&5u32));
    /// ```
    #[cfg_attr(creusot, ensures(result == response_builder_extensions(self)))]
    pub fn extensions_ref(&self) -> Option<&Extensions> {
        self.inner.as_ref().ok().map(|h| &h.extensions)
    }

    /// Get a mutable reference to the extensions for this response builder.
    ///
    /// If the builder has an error, this returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::Response;
    /// let mut res = Response::builder().extension("My Extension");
    /// let mut extensions = res.extensions_mut().unwrap();
    /// assert_eq!(extensions.get::<&'static str>(), Some(&"My Extension"));
    /// extensions.insert(5u32);
    /// assert_eq!(extensions.get::<u32>(), Some(&5u32));
    /// ```
    #[cfg_attr(creusot, ensures(match result {
        Some(reference) => Some(&*reference) == response_builder_extensions(&*self),
        None => response_builder_extensions(&*self) == None,
    }))]
    #[cfg_attr(creusot, ensures(match result {
        Some(reference) => Some(&^reference) == response_builder_extensions(&^self),
        None => response_builder_extensions(&^self) == None,
    }))]
    #[cfg_attr(creusot, ensures(response_builder_error(&^self) == response_builder_error(&*self)))]
    #[cfg_attr(creusot, ensures(response_builder_status(&^self) == response_builder_status(&*self)))]
    #[cfg_attr(creusot, ensures(response_builder_version(&^self) == response_builder_version(&*self)))]
    #[cfg_attr(creusot, ensures(response_builder_headers(&^self) == response_builder_headers(&*self)))]
    pub fn extensions_mut(&mut self) -> Option<&mut Extensions> {
        match &mut self.inner {
            Ok(head) => Some(&mut head.extensions),
            Err(_) => None,
        }
    }

    /// "Consumes" this builder, using the provided `body` to return a
    /// constructed `Response`.
    ///
    /// # Errors
    ///
    /// This function may return an error if any previously configured argument
    /// failed to parse or get converted to the internal representation. For
    /// example if an invalid `head` was specified via `header("Foo",
    /// "Bar\r\n")` the error will be returned when this function is called
    /// rather than when `header` was called.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let response = Response::builder()
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, ensures(response_result_head(&result) == response_builder_parts(&self)))]
    #[cfg_attr(creusot, ensures(response_result_body(&result) == if response_builder_is_valid(&self) { Some(&body) } else { None }))]
    #[cfg_attr(creusot, ensures(response_result_error(&result) == response_builder_error(&self)))]
    pub fn body<T>(self, body: T) -> Result<Response<T>> {
        match self.inner {
            Ok(head) => Ok(Response { head, body }),
            Err(error) => Err(error),
        }
    }

    // private

    #[requires(match self.inner {
        Ok(head) => func.precondition((head,)),
        Err(_) => true,
    })]
    #[ensures(match self.inner {
        Ok(head) => func.postcondition_once((head,), result.inner),
        Err(_) => response_builder_error(&result) == response_builder_error(&self),
    })]
    fn and_then<F>(self, func: F) -> Self
    where
        F: FnOnce(Parts) -> Result<Parts>,
    {
        let inner = match self.inner {
            Ok(head) => func(head),
            Err(error) => Err(error),
        };
        Builder {
            inner,
        }
    }
}

impl Default for Builder {
    #[inline]
    #[cfg_attr(creusot, ensures(response_builder_is_valid(&result)))]
    #[cfg_attr(all(creusot, any(not(http_composition_leaf), http_composition_uri_defaults_leaf)), ensures(
        response_builder_has_default_core_fields(&result)
    ))]
    fn default() -> Builder {
        Builder {
            inner: Ok(Parts::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_map_a_body_from_one_type_to_another() {
        let response = Response::builder().body("some string").unwrap();
        let mapped_response = response.map(|s| {
            assert_eq!(s, "some string");
            123u32
        });
        assert_eq!(mapped_response.body(), &123u32);
    }
}
