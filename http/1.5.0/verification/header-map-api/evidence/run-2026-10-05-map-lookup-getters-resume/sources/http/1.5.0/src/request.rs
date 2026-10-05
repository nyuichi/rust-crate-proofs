//! HTTP request types.
//!
//! This module contains structs related to HTTP requests, notably the
//! `Request` type itself as well as a builder to create requests. Typically
//! you'll import the `http::Request` type rather than reaching into this
//! module itself.
//!
//! # Examples
//!
//! Creating a `Request` to send
//!
//! ```no_run
//! use http::{Request, Response};
//!
//! let mut request = Request::builder()
//!     .uri("https://www.rust-lang.org/")
//!     .header("User-Agent", "my-awesome-agent/1.0");
//!
//! if needs_awesome_header() {
//!     request = request.header("Awesome", "yes");
//! }
//!
//! let response = send(request.body(()).unwrap());
//!
//! # fn needs_awesome_header() -> bool {
//! #     true
//! # }
//! #
//! fn send(req: Request<()>) -> Response<()> {
//!     // ...
//! # panic!()
//! }
//! ```
//!
//! Inspecting a request to see what was sent.
//!
//! ```
//! use http::{Request, Response, StatusCode};
//!
//! fn respond_to(req: Request<()>) -> http::Result<Response<()>> {
//!     if req.uri() != "/awesome-url" {
//!         return Response::builder()
//!             .status(StatusCode::NOT_FOUND)
//!             .body(())
//!     }
//!
//!     let has_awesome_header = req.headers().contains_key("Awesome");
//!     let body = req.body();
//!
//!     // ...
//! # panic!()
//! }
//! ```

use std::any::Any;
use std::convert::TryInto;
use std::fmt;

#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite, requires, DeepModel, Invariant, View};

use crate::header::{HeaderMap, HeaderName, HeaderValue};
use crate::method::Method;
use crate::version::Version;
use crate::{Extensions, Result, Uri};

#[cfg(creusot)]
use creusot_std::std::ops::{FnExt as _, FnOnceExt as _};

/// Represents an HTTP request.
///
/// An HTTP request consists of a head and a potentially optional body. The body
/// component is generic, enabling arbitrary types to represent the HTTP body.
/// For example, the body could be `Vec<u8>`, a `Stream` of byte chunks, or a
/// value that has been deserialized.
///
/// # Examples
///
/// Creating a `Request` to send
///
/// ```no_run
/// use http::{Request, Response};
///
/// let mut request = Request::builder()
///     .uri("https://www.rust-lang.org/")
///     .header("User-Agent", "my-awesome-agent/1.0");
///
/// if needs_awesome_header() {
///     request = request.header("Awesome", "yes");
/// }
///
/// let response = send(request.body(()).unwrap());
///
/// # fn needs_awesome_header() -> bool {
/// #     true
/// # }
/// #
/// fn send(req: Request<()>) -> Response<()> {
///     // ...
/// # panic!()
/// }
/// ```
///
/// Inspecting a request to see what was sent.
///
/// ```
/// use http::{Request, Response, StatusCode};
///
/// fn respond_to(req: Request<()>) -> http::Result<Response<()>> {
///     if req.uri() != "/awesome-url" {
///         return Response::builder()
///             .status(StatusCode::NOT_FOUND)
///             .body(())
///     }
///
///     let has_awesome_header = req.headers().contains_key("Awesome");
///     let body = req.body();
///
///     // ...
/// # panic!()
/// }
/// ```
///
/// Deserialize a request of bytes via json:
///
/// ```
/// use http::Request;
/// use serde::de;
///
/// fn deserialize<T>(req: Request<Vec<u8>>) -> serde_json::Result<Request<T>>
///     where for<'de> T: de::Deserialize<'de>,
/// {
///     let (parts, body) = req.into_parts();
///     let body = serde_json::from_slice(&body)?;
///     Ok(Request::from_parts(parts, body))
/// }
/// #
/// # fn main() {}
/// ```
///
/// Or alternatively, serialize the body of a request to json
///
/// ```
/// use http::Request;
/// use serde::ser;
///
/// fn serialize<T>(req: Request<T>) -> serde_json::Result<Request<Vec<u8>>>
///     where T: ser::Serialize,
/// {
///     let (parts, body) = req.into_parts();
///     let body = serde_json::to_vec(&body)?;
///     Ok(Request::from_parts(parts, body))
/// }
/// #
/// # fn main() {}
/// ```
#[derive(Clone)]
pub struct Request<T> {
    head: Parts,
    body: T,
}

/// Component parts of an HTTP `Request`
///
/// The HTTP request head consists of a method, uri, version, and a set of
/// header fields.
#[allow(unexpected_cfgs)]
pub struct Parts {
    /// The request's method
    pub method: Method,

    /// The request's URI
    pub uri: Uri,

    /// The request's version
    pub version: Version,

    /// The request's headers
    pub headers: HeaderMap<HeaderValue>,

    /// The request's extensions
    pub extensions: Extensions,

    _priv: (),
}

impl Clone for Parts {
    fn clone(&self) -> Parts {
        Parts {
            method: self.method.clone(),
            uri: self.uri.clone(),
            version: self.version.clone(),
            headers: self.headers.clone(),
            extensions: self.extensions.clone(),
            _priv: self._priv,
        }
    }
}

/// An HTTP request builder
///
/// This type can be used to construct an instance of `Request`
/// through a builder-like pattern.
#[cfg_attr(not(any(http_composition_leaf, http_builder_entrypoints_leaf)), derive(Debug))]
pub struct Builder {
    inner: Result<Parts>,
}

/// A specification-only projection of a request's head.
#[cfg(creusot)]
#[logic]
pub fn request_head<T>(request: Request<T>) -> Parts {
    request.head
}

/// A specification-only projection of a request's body.
#[cfg(creusot)]
#[logic]
pub fn request_body<T>(request: Request<T>) -> T {
    request.body
}

/// Whether a request builder currently contains a valid head.
#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn request_builder_is_valid(builder: &Builder) -> bool {
    match &builder.inner {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// The method, URI, and version fields initialized by the default builder.
#[cfg(all(creusot, any(not(http_composition_leaf), http_composition_uri_defaults_leaf)))]
#[doc(hidden)]
#[logic]
pub fn request_builder_has_default_core_fields(builder: &Builder) -> bool {
    pearlite! {
        match request_builder_parts(builder) {
            Some(parts) => {
                parts.method.deep_model()
                    == crate::method::method_model_text(crate::method::MethodModel::Get)
                    && crate::uri::uri_is_default(parts.uri@)
                    && parts.version.deep_model() == 11
            }
            None => false,
        }
    }
}

/// The exact stored error, including its concrete payload, when a builder is
/// in the error state.
#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn request_builder_error<'a>(builder: &'a Builder) -> Option<crate::ErrorModelRef<'a>> {
    match &builder.inner {
        Ok(_) => None,
        Err(error) => Some(crate::error_model_ref(error)),
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn request_builder_error_is_from<E>(builder: &Builder, source: E) -> bool
where
    E: Into<crate::Error>,
{
    match &builder.inner {
        Ok(_) => false,
        Err(error) => <E as Into<crate::Error>>::into.postcondition((source,), *error),
    }
}

/// Exact fixed-method/URI outcome relation for the request shortcuts. The
/// URI conversion's original error payload is retained in the builder.
#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn request_fixed_method_post<T>(builder: &Builder, method: &Method, uri: &T) -> bool
where
    T: TryInto<Uri>,
    <T as TryInto<Uri>>::Error: Into<crate::Error>,
{
    pearlite! {
        exists<method_result: std::result::Result<Method, <Method as TryInto<Method>>::Error>>
            <Method as TryInto<Method>>::try_into.postcondition((*method,), method_result)
                && match method_result {
                    Ok(converted_method) => converted_method == *method
                        && exists<uri_result: std::result::Result<Uri, <T as TryInto<Uri>>::Error>>
                            <T as TryInto<Uri>>::try_into.postcondition((*uri,), uri_result)
                                && match uri_result {
                                    Ok(converted_uri) => request_builder_is_valid(builder)
                                        && request_builder_method(builder) == Some(method)
                                        && request_builder_uri(builder) == Some(&converted_uri)
                                        && request_builder_error(builder) == None,
                                    Err(conversion_error) => !request_builder_is_valid(builder)
                                        && request_builder_error_is_from(builder, conversion_error)
                                        && request_builder_method(builder) == None
                                        && request_builder_uri(builder) == None
                                        && request_builder_version(builder) == None
                                        && request_builder_headers(builder) == None
                                        && request_builder_extensions(builder) == None,
                                },
                    Err(conversion_error) => !request_builder_is_valid(builder)
                        && request_builder_error_is_from(builder, conversion_error)
                        && request_builder_method(builder) == None
                        && request_builder_uri(builder) == None
                        && request_builder_version(builder) == None
                        && request_builder_headers(builder) == None
                        && request_builder_extensions(builder) == None,
                }
    }
}

/// The fixed-method shortcut result when the built-in value is identified by
/// its exact method text. This lets callers state the method without
/// translating an associated constant across the Method module boundary.
#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn request_fixed_method_model_post<T>(
    builder: &Builder,
    method_text: creusot_std::logic::Seq<u8>,
    uri: &T,
) -> bool
where
    T: TryInto<Uri>,
    <T as TryInto<Uri>>::Error: Into<crate::Error>,
{
    pearlite! {
        exists<source_method: Method>
            source_method.invariant()
                && source_method.deep_model() == method_text
                && exists<method_result: std::result::Result<Method, <Method as TryInto<Method>>::Error>>
                    <Method as TryInto<Method>>::try_into.postcondition((source_method,), method_result)
                        && match method_result {
                            Ok(converted_method) => converted_method.deep_model() == method_text
                                && exists<uri_result: std::result::Result<Uri, <T as TryInto<Uri>>::Error>>
                                    <T as TryInto<Uri>>::try_into.postcondition((*uri,), uri_result)
                                        && match uri_result {
                                            Ok(converted_uri) => request_builder_is_valid(builder)
                                                && match request_builder_method(builder) {
                                                    Some(stored) => stored.deep_model() == method_text,
                                                    None => false,
                                                }
                                                && request_builder_uri(builder) == Some(&converted_uri)
                                                && request_builder_error(builder) == None,
                                            Err(conversion_error) => !request_builder_is_valid(builder)
                                                && request_builder_error_is_from(builder, conversion_error)
                                                && request_builder_method(builder) == None
                                                && request_builder_uri(builder) == None
                                                && request_builder_version(builder) == None
                                                && request_builder_headers(builder) == None
                                                && request_builder_extensions(builder) == None,
                                        },
                            Err(conversion_error) => !request_builder_is_valid(builder)
                                && request_builder_error_is_from(builder, conversion_error)
                                && request_builder_method(builder) == None
                                && request_builder_uri(builder) == None
                                && request_builder_version(builder) == None
                                && request_builder_headers(builder) == None
                                && request_builder_extensions(builder) == None,
                        }
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn request_builder_method<'a>(builder: &'a Builder) -> Option<&'a Method> {
    match &builder.inner {
        Ok(head) => Some(&head.method),
        Err(_) => None,
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn request_builder_uri<'a>(builder: &'a Builder) -> Option<&'a Uri> {
    match &builder.inner {
        Ok(head) => Some(&head.uri),
        Err(_) => None,
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn request_builder_version<'a>(builder: &'a Builder) -> Option<&'a Version> {
    match &builder.inner {
        Ok(head) => Some(&head.version),
        Err(_) => None,
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn request_builder_headers<'a>(builder: &'a Builder) -> Option<&'a HeaderMap<HeaderValue>> {
    match &builder.inner {
        Ok(head) => Some(&head.headers),
        Err(_) => None,
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn request_builder_extensions<'a>(builder: &'a Builder) -> Option<&'a Extensions> {
    match &builder.inner {
        Ok(head) => Some(&head.extensions),
        Err(_) => None,
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn request_builder_parts<'a>(builder: &'a Builder) -> Option<&'a Parts> {
    match &builder.inner {
        Ok(head) => Some(head),
        Err(_) => None,
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn request_result_head<'a, T>(outcome: &'a Result<Request<T>>) -> Option<&'a Parts> {
    match outcome {
        Ok(request) => Some(&request.head),
        Err(_) => None,
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn request_result_body<'a, T>(outcome: &'a Result<Request<T>>) -> Option<&'a T> {
    match outcome {
        Ok(request) => Some(&request.body),
        Err(_) => None,
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn request_result_error<'a, T>(outcome: &'a Result<Request<T>>) -> Option<crate::ErrorModelRef<'a>> {
    match outcome {
        Ok(_) => None,
        Err(error) => Some(crate::error_model_ref(error)),
    }
}

#[allow(unexpected_cfgs)]
#[cfg(any(not(http_composition_leaf), http_builder_entrypoints_leaf))]
impl Request<()> {
    /// Creates a new builder-style object to manufacture a `Request`
    ///
    /// This method returns an instance of `Builder` which can be used to
    /// create a `Request`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let request = Request::builder()
    ///     .method("GET")
    ///     .uri("https://www.rust-lang.org/")
    ///     .header("X-Custom-Foo", "Bar")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[inline]
    #[cfg_attr(creusot, ensures(request_builder_is_valid(&result)))]
    #[cfg_attr(all(creusot, any(not(http_composition_leaf), http_composition_uri_defaults_leaf)), ensures(
        request_builder_has_default_core_fields(&result)
    ))]
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Creates a new `Builder` initialized with a GET method and the given URI.
    ///
    /// This method returns an instance of `Builder` which can be used to
    /// create a `Request`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::*;
    ///
    /// let request = Request::get("https://www.rust-lang.org/")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, requires(
        <T as TryInto<Uri>>::try_into.precondition((uri,))
            && forall<uri_error: <T as TryInto<Uri>>::Error>
                <T as TryInto<Uri>>::try_into.postcondition((uri,), Err(uri_error))
                    ==> <<T as TryInto<Uri>>::Error as Into<crate::Error>>::into.precondition((uri_error,))
    ))]
    #[cfg_attr(creusot, ensures(request_fixed_method_model_post(
        &result,
        crate::method::method_model_text(crate::method::MethodModel::Get),
        &uri,
    )))]
    pub fn get<T>(uri: T) -> Builder
    where
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: Into<crate::Error>,
    {
        Builder::new().method(Method::builder_get()).uri(uri)
    }

    /// Creates a new `Builder` initialized with a PUT method and the given URI.
    ///
    /// This method returns an instance of `Builder` which can be used to
    /// create a `Request`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::*;
    ///
    /// let request = Request::put("https://www.rust-lang.org/")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, requires(
        <T as TryInto<Uri>>::try_into.precondition((uri,))
            && forall<uri_error: <T as TryInto<Uri>>::Error>
                <T as TryInto<Uri>>::try_into.postcondition((uri,), Err(uri_error))
                    ==> <<T as TryInto<Uri>>::Error as Into<crate::Error>>::into.precondition((uri_error,))
    ))]
    #[cfg_attr(creusot, ensures(request_fixed_method_model_post(
        &result,
        crate::method::method_model_text(crate::method::MethodModel::Put),
        &uri,
    )))]
    pub fn put<T>(uri: T) -> Builder
    where
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: Into<crate::Error>,
    {
        Builder::new().method(Method::builder_put()).uri(uri)
    }

    /// Creates a new `Builder` initialized with a POST method and the given URI.
    ///
    /// This method returns an instance of `Builder` which can be used to
    /// create a `Request`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::*;
    ///
    /// let request = Request::post("https://www.rust-lang.org/")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, requires(
        <T as TryInto<Uri>>::try_into.precondition((uri,))
            && forall<uri_error: <T as TryInto<Uri>>::Error>
                <T as TryInto<Uri>>::try_into.postcondition((uri,), Err(uri_error))
                    ==> <<T as TryInto<Uri>>::Error as Into<crate::Error>>::into.precondition((uri_error,))
    ))]
    #[cfg_attr(creusot, ensures(request_fixed_method_model_post(
        &result,
        crate::method::method_model_text(crate::method::MethodModel::Post),
        &uri,
    )))]
    pub fn post<T>(uri: T) -> Builder
    where
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: Into<crate::Error>,
    {
        Builder::new().method(Method::builder_post()).uri(uri)
    }

    /// Creates a new `Builder` initialized with a DELETE method and the given URI.
    ///
    /// This method returns an instance of `Builder` which can be used to
    /// create a `Request`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::*;
    ///
    /// let request = Request::delete("https://www.rust-lang.org/")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, requires(
        <T as TryInto<Uri>>::try_into.precondition((uri,))
            && forall<uri_error: <T as TryInto<Uri>>::Error>
                <T as TryInto<Uri>>::try_into.postcondition((uri,), Err(uri_error))
                    ==> <<T as TryInto<Uri>>::Error as Into<crate::Error>>::into.precondition((uri_error,))
    ))]
    #[cfg_attr(creusot, ensures(request_fixed_method_model_post(
        &result,
        crate::method::method_model_text(crate::method::MethodModel::Delete),
        &uri,
    )))]
    pub fn delete<T>(uri: T) -> Builder
    where
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: Into<crate::Error>,
    {
        Builder::new().method(Method::builder_delete()).uri(uri)
    }

    /// Creates a new `Builder` initialized with an OPTIONS method and the given URI.
    ///
    /// This method returns an instance of `Builder` which can be used to
    /// create a `Request`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::*;
    ///
    /// let request = Request::options("https://www.rust-lang.org/")
    ///     .body(())
    ///     .unwrap();
    /// # assert_eq!(*request.method(), Method::OPTIONS);
    /// ```
    #[cfg_attr(creusot, requires(
        <T as TryInto<Uri>>::try_into.precondition((uri,))
            && forall<uri_error: <T as TryInto<Uri>>::Error>
                <T as TryInto<Uri>>::try_into.postcondition((uri,), Err(uri_error))
                    ==> <<T as TryInto<Uri>>::Error as Into<crate::Error>>::into.precondition((uri_error,))
    ))]
    #[cfg_attr(creusot, ensures(request_fixed_method_model_post(
        &result,
        crate::method::method_model_text(crate::method::MethodModel::Options),
        &uri,
    )))]
    pub fn options<T>(uri: T) -> Builder
    where
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: Into<crate::Error>,
    {
        Builder::new().method(Method::builder_options()).uri(uri)
    }

    /// Creates a new `Builder` initialized with a HEAD method and the given URI.
    ///
    /// This method returns an instance of `Builder` which can be used to
    /// create a `Request`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::*;
    ///
    /// let request = Request::head("https://www.rust-lang.org/")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, requires(
        <T as TryInto<Uri>>::try_into.precondition((uri,))
            && forall<uri_error: <T as TryInto<Uri>>::Error>
                <T as TryInto<Uri>>::try_into.postcondition((uri,), Err(uri_error))
                    ==> <<T as TryInto<Uri>>::Error as Into<crate::Error>>::into.precondition((uri_error,))
    ))]
    #[cfg_attr(creusot, ensures(request_fixed_method_model_post(
        &result,
        crate::method::method_model_text(crate::method::MethodModel::Head),
        &uri,
    )))]
    pub fn head<T>(uri: T) -> Builder
    where
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: Into<crate::Error>,
    {
        Builder::new().method(Method::builder_head()).uri(uri)
    }

    /// Creates a new `Builder` initialized with a CONNECT method and the given URI.
    ///
    /// This method returns an instance of `Builder` which can be used to
    /// create a `Request`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::*;
    ///
    /// let request = Request::connect("https://www.rust-lang.org/")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, requires(
        <T as TryInto<Uri>>::try_into.precondition((uri,))
            && forall<uri_error: <T as TryInto<Uri>>::Error>
                <T as TryInto<Uri>>::try_into.postcondition((uri,), Err(uri_error))
                    ==> <<T as TryInto<Uri>>::Error as Into<crate::Error>>::into.precondition((uri_error,))
    ))]
    #[cfg_attr(creusot, ensures(request_fixed_method_model_post(
        &result,
        crate::method::method_model_text(crate::method::MethodModel::Connect),
        &uri,
    )))]
    pub fn connect<T>(uri: T) -> Builder
    where
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: Into<crate::Error>,
    {
        Builder::new().method(Method::builder_connect()).uri(uri)
    }

    /// Creates a new `Builder` initialized with a PATCH method and the given URI.
    ///
    /// This method returns an instance of `Builder` which can be used to
    /// create a `Request`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::*;
    ///
    /// let request = Request::patch("https://www.rust-lang.org/")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, requires(
        <T as TryInto<Uri>>::try_into.precondition((uri,))
            && forall<uri_error: <T as TryInto<Uri>>::Error>
                <T as TryInto<Uri>>::try_into.postcondition((uri,), Err(uri_error))
                    ==> <<T as TryInto<Uri>>::Error as Into<crate::Error>>::into.precondition((uri_error,))
    ))]
    #[cfg_attr(creusot, ensures(request_fixed_method_model_post(
        &result,
        crate::method::method_model_text(crate::method::MethodModel::Patch),
        &uri,
    )))]
    pub fn patch<T>(uri: T) -> Builder
    where
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: Into<crate::Error>,
    {
        Builder::new().method(Method::builder_patch()).uri(uri)
    }

    /// Creates a new `Builder` initialized with a TRACE method and the given URI.
    ///
    /// This method returns an instance of `Builder` which can be used to
    /// create a `Request`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::*;
    ///
    /// let request = Request::trace("https://www.rust-lang.org/")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, requires(
        <T as TryInto<Uri>>::try_into.precondition((uri,))
            && forall<uri_error: <T as TryInto<Uri>>::Error>
                <T as TryInto<Uri>>::try_into.postcondition((uri,), Err(uri_error))
                    ==> <<T as TryInto<Uri>>::Error as Into<crate::Error>>::into.precondition((uri_error,))
    ))]
    #[cfg_attr(creusot, ensures(request_fixed_method_model_post(
        &result,
        crate::method::method_model_text(crate::method::MethodModel::Trace),
        &uri,
    )))]
    pub fn trace<T>(uri: T) -> Builder
    where
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: Into<crate::Error>,
    {
        Builder::new().method(Method::builder_trace()).uri(uri)
    }

    // This is purposefully excluded because of potential conflict with the
    // URI query.
    // pub fn query() -> Builder
}

impl<T> Request<T> {
    /// Creates a new blank `Request` with the body
    ///
    /// The component parts of this request will be set to their default, e.g.
    /// the GET method, no headers, etc.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let request = Request::new("hello world");
    ///
    /// assert_eq!(*request.method(), Method::GET);
    /// assert_eq!(*request.body(), "hello world");
    /// ```
    #[inline]
    pub fn new(body: T) -> Request<T> {
        Request {
            head: Parts::new(),
            body,
        }
    }

    /// Creates a new `Request` with the given components parts and body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let request = Request::new("hello world");
    /// let (mut parts, body) = request.into_parts();
    /// parts.method = Method::POST;
    ///
    /// let request = Request::from_parts(parts, body);
    /// ```
    #[inline]
    #[ensures(request_head(result) == parts && request_body(result) == body)]
    pub fn from_parts(parts: Parts, body: T) -> Request<T> {
        Request { head: parts, body }
    }

    /// Returns a reference to the associated HTTP method.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let request: Request<()> = Request::default();
    /// assert_eq!(*request.method(), Method::GET);
    /// ```
    #[inline]
    #[ensures(*result == request_head(*self).method)]
    pub fn method(&self) -> &Method {
        &self.head.method
    }

    /// Returns a mutable reference to the associated HTTP method.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let mut request: Request<()> = Request::default();
    /// *request.method_mut() = Method::PUT;
    /// assert_eq!(*request.method(), Method::PUT);
    /// ```
    #[inline]
    #[ensures(*result == request_head(*self).method)]
    #[ensures(^result == request_head(^self).method)]
    #[ensures(request_head(^self).uri == request_head(*self).uri)]
    #[ensures(request_head(^self).version == request_head(*self).version)]
    #[ensures(request_head(^self).headers == request_head(*self).headers)]
    #[ensures(request_head(^self).extensions == request_head(*self).extensions)]
    #[ensures(request_body(^self) == request_body(*self))]
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.head.method
    }

    /// Returns a reference to the associated URI.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let request: Request<()> = Request::default();
    /// assert_eq!(*request.uri(), *"/");
    /// ```
    #[inline]
    #[ensures(*result == request_head(*self).uri)]
    pub fn uri(&self) -> &Uri {
        &self.head.uri
    }

    /// Returns a mutable reference to the associated URI.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let mut request: Request<()> = Request::default();
    /// *request.uri_mut() = "/hello".parse().unwrap();
    /// assert_eq!(*request.uri(), *"/hello");
    /// ```
    #[inline]
    #[ensures(*result == request_head(*self).uri)]
    #[ensures(^result == request_head(^self).uri)]
    #[ensures(request_head(^self).method == request_head(*self).method)]
    #[ensures(request_head(^self).version == request_head(*self).version)]
    #[ensures(request_head(^self).headers == request_head(*self).headers)]
    #[ensures(request_head(^self).extensions == request_head(*self).extensions)]
    #[ensures(request_body(^self) == request_body(*self))]
    pub fn uri_mut(&mut self) -> &mut Uri {
        &mut self.head.uri
    }

    /// Returns the associated version.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let request: Request<()> = Request::default();
    /// assert_eq!(request.version(), Version::HTTP_11);
    /// ```
    #[inline]
    #[ensures(result == request_head(*self).version)]
    pub fn version(&self) -> Version {
        self.head.version
    }

    /// Returns a mutable reference to the associated version.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let mut request: Request<()> = Request::default();
    /// *request.version_mut() = Version::HTTP_2;
    /// assert_eq!(request.version(), Version::HTTP_2);
    /// ```
    #[inline]
    #[ensures(*result == request_head(*self).version)]
    #[ensures(^result == request_head(^self).version)]
    #[ensures(request_head(^self).method == request_head(*self).method)]
    #[ensures(request_head(^self).uri == request_head(*self).uri)]
    #[ensures(request_head(^self).headers == request_head(*self).headers)]
    #[ensures(request_head(^self).extensions == request_head(*self).extensions)]
    #[ensures(request_body(^self) == request_body(*self))]
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.head.version
    }

    /// Returns a reference to the associated header field map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let request: Request<()> = Request::default();
    /// assert!(request.headers().is_empty());
    /// ```
    #[inline]
    #[ensures(*result == request_head(*self).headers)]
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
    /// let mut request: Request<()> = Request::default();
    /// request.headers_mut().insert(HOST, HeaderValue::from_static("world"));
    /// assert!(!request.headers().is_empty());
    /// ```
    #[inline]
    #[ensures(*result == request_head(*self).headers)]
    #[ensures(^result == request_head(^self).headers)]
    #[ensures(request_head(^self).method == request_head(*self).method)]
    #[ensures(request_head(^self).uri == request_head(*self).uri)]
    #[ensures(request_head(^self).version == request_head(*self).version)]
    #[ensures(request_head(^self).extensions == request_head(*self).extensions)]
    #[ensures(request_body(^self) == request_body(*self))]
    pub fn headers_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
        &mut self.head.headers
    }

    /// Returns a reference to the associated extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let request: Request<()> = Request::default();
    /// assert!(request.extensions().get::<i32>().is_none());
    /// ```
    #[inline]
    #[ensures(*result == request_head(*self).extensions)]
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
    /// let mut request: Request<()> = Request::default();
    /// request.extensions_mut().insert("hello");
    /// assert_eq!(request.extensions().get(), Some(&"hello"));
    /// ```
    #[inline]
    #[ensures(*result == request_head(*self).extensions)]
    #[ensures(^result == request_head(^self).extensions)]
    #[ensures(request_head(^self).method == request_head(*self).method)]
    #[ensures(request_head(^self).uri == request_head(*self).uri)]
    #[ensures(request_head(^self).version == request_head(*self).version)]
    #[ensures(request_head(^self).headers == request_head(*self).headers)]
    #[ensures(request_body(^self) == request_body(*self))]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.head.extensions
    }

    /// Returns a reference to the associated HTTP body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let request: Request<String> = Request::default();
    /// assert!(request.body().is_empty());
    /// ```
    #[inline]
    #[ensures(*result == request_body(*self))]
    pub fn body(&self) -> &T {
        &self.body
    }

    /// Returns a mutable reference to the associated HTTP body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let mut request: Request<String> = Request::default();
    /// request.body_mut().push_str("hello world");
    /// assert!(!request.body().is_empty());
    /// ```
    #[inline]
    #[ensures(*result == request_body(*self))]
    #[ensures(^result == request_body(^self))]
    #[ensures(request_head(^self) == request_head(*self))]
    pub fn body_mut(&mut self) -> &mut T {
        &mut self.body
    }

    /// Consumes the request, returning just the body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::Request;
    /// let request = Request::new(10);
    /// let body = request.into_body();
    /// assert_eq!(body, 10);
    /// ```
    #[inline]
    #[ensures(result == request_body(self))]
    pub fn into_body(self) -> T {
        self.body
    }

    /// Consumes the request returning the head and body parts.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let request = Request::new(());
    /// let (parts, body) = request.into_parts();
    /// assert_eq!(parts.method, Method::GET);
    /// ```
    #[inline]
    #[ensures(result.0 == request_head(self) && result.1 == request_body(self))]
    pub fn into_parts(self) -> (Parts, T) {
        (self.head, self.body)
    }

    /// Consumes the request returning a new request with body mapped to the
    /// return type of the passed in function.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    /// let request = Request::builder().body("some string").unwrap();
    /// let mapped_request: Request<&[u8]> = request.map(|b| {
    ///   assert_eq!(b, "some string");
    ///   b.as_bytes()
    /// });
    /// assert_eq!(mapped_request.body(), &"some string".as_bytes());
    /// ```
    #[inline]
    #[requires(f.precondition((request_body(self),)))]
    #[ensures(request_head(result) == request_head(self))]
    #[ensures(f.postcondition_once((request_body(self),), request_body(result)))]
    pub fn map<F, U>(self, f: F) -> Request<U>
    where
        F: FnOnce(T) -> U,
    {
        Request {
            body: f(self.body),
            head: self.head,
        }
    }
}

impl<T: Default> Default for Request<T> {
    fn default() -> Request<T> {
        Request::new(T::default())
    }
}

impl<T: fmt::Debug> fmt::Debug for Request<T> {
    #[cfg_attr(creusot, ensures(
        creusot_std::std::fmt::formatter_extends(f.deep_model(), (^f).deep_model())
    ))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Request")
            .field("method", self.method())
            .field("uri", self.uri())
            .field("version", &self.version())
            .field("headers", self.headers())
            // omits Extensions because not useful
            .field("body", self.body())
            .finish()
    }
}

impl Parts {
    /// Creates a new default instance of `Parts`
    #[cfg_attr(creusot, ensures(
        result.method.deep_model()
            == crate::method::method_model_text(crate::method::MethodModel::Get)
    ))]
    #[cfg_attr(all(creusot, any(not(http_composition_leaf), http_composition_uri_defaults_leaf)), ensures(
        crate::uri::uri_is_default(result.uri@)
    ))]
    #[cfg_attr(all(creusot, any(not(http_composition_leaf), http_composition_uri_defaults_leaf)), ensures(
        result.version.deep_model() == 11
    ))]
    fn new() -> Parts {
        Parts {
            method: Method::default(),
            uri: Uri::default(),
            version: Version::default(),
            headers: HeaderMap::default(),
            extensions: Extensions::default(),
            _priv: (),
        }
    }
}

impl fmt::Debug for Parts {
    #[cfg_attr(creusot, ensures(
        creusot_std::std::fmt::formatter_extends(f.deep_model(), (^f).deep_model())
    ))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parts")
            .field("method", &self.method)
            .field("uri", &self.uri)
            .field("version", &self.version)
            .field("headers", &self.headers)
            // omits Extensions because not useful
            // omits _priv because not useful
            .finish()
    }
}

impl Builder {
    /// Creates a new default instance of `Builder` to construct a `Request`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let req = request::Builder::new()
    ///     .method("POST")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[inline]
    #[cfg_attr(creusot, ensures(request_builder_is_valid(&result)))]
    #[cfg_attr(all(creusot, any(not(http_composition_leaf), http_composition_uri_defaults_leaf)), ensures(
        request_builder_has_default_core_fields(&result)
    ))]
    pub fn new() -> Builder {
        Builder::default()
    }

    /// Set the HTTP method for this request.
    ///
    /// By default this is `GET`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let req = Request::builder()
    ///     .method("POST")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, requires(!request_builder_is_valid(&self) || (
        <T as TryInto<Method>>::try_into.precondition((method,))
        && forall<e: <T as TryInto<Method>>::Error>
            <T as TryInto<Method>>::try_into.postcondition((method,), Err(e))
                ==> <<T as TryInto<Method>>::Error as Into<crate::Error>>::into.precondition((e,))
    )))]
    #[cfg_attr(creusot, ensures(!request_builder_is_valid(&self) ==> request_builder_error(&result) == request_builder_error(&self)))]
    #[cfg_attr(creusot, ensures(request_builder_is_valid(&self) ==> exists<converted: std::result::Result<Method, <T as TryInto<Method>>::Error>>
        <T as TryInto<Method>>::try_into.postcondition((method,), converted)
        && match converted {
            Ok(new_method) => request_builder_is_valid(&result)
                && request_builder_method(&result) == Some(&new_method)
                && request_builder_error(&result) == None
                && request_builder_uri(&result) == request_builder_uri(&self)
                && request_builder_version(&result) == request_builder_version(&self)
                && request_builder_headers(&result) == request_builder_headers(&self)
                && request_builder_extensions(&result) == request_builder_extensions(&self),
            Err(conversion_error) => !request_builder_is_valid(&result)
                && request_builder_error_is_from(&result, conversion_error)
                && request_builder_method(&result) == None
                && request_builder_uri(&result) == None
                && request_builder_version(&result) == None
                && request_builder_headers(&result) == None
                && request_builder_extensions(&result) == None,
        }
    ))]
    pub fn method<T>(self, method: T) -> Builder
    where
        T: TryInto<Method>,
        <T as TryInto<Method>>::Error: Into<crate::Error>,
    {
        self.and_then(move |mut head| {
            let method = match method.try_into() {
                Ok(method) => method,
                Err(error) => return Err(error.into()),
            };
            head.method = method;
            Ok(head)
        })
    }

    /// Get the HTTP Method for this request.
    ///
    /// By default this is `GET`. If builder has error, returns None.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let mut req = Request::builder();
    /// assert_eq!(req.method_ref(),Some(&Method::GET));
    ///
    /// req = req.method("POST");
    /// assert_eq!(req.method_ref(),Some(&Method::POST));
    /// ```
    #[cfg_attr(creusot, ensures(result == request_builder_method(self)))]
    pub fn method_ref(&self) -> Option<&Method> {
        self.inner.as_ref().ok().map(|h| &h.method)
    }

    /// Set the URI for this request.
    ///
    /// By default this is `/`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let req = Request::builder()
    ///     .uri("https://www.rust-lang.org/")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, requires(!request_builder_is_valid(&self) || (
        <T as TryInto<Uri>>::try_into.precondition((uri,))
        && forall<e: <T as TryInto<Uri>>::Error>
            <T as TryInto<Uri>>::try_into.postcondition((uri,), Err(e))
                ==> <<T as TryInto<Uri>>::Error as Into<crate::Error>>::into.precondition((e,))
    )))]
    #[cfg_attr(creusot, ensures(!request_builder_is_valid(&self) ==> request_builder_error(&result) == request_builder_error(&self)))]
    #[cfg_attr(creusot, ensures(request_builder_is_valid(&self) ==> exists<converted: std::result::Result<Uri, <T as TryInto<Uri>>::Error>>
        <T as TryInto<Uri>>::try_into.postcondition((uri,), converted)
        && match converted {
            Ok(new_uri) => request_builder_is_valid(&result)
                && request_builder_uri(&result) == Some(&new_uri)
                && request_builder_error(&result) == None
                && request_builder_method(&result) == request_builder_method(&self)
                && request_builder_version(&result) == request_builder_version(&self)
                && request_builder_headers(&result) == request_builder_headers(&self)
                && request_builder_extensions(&result) == request_builder_extensions(&self),
            Err(conversion_error) => !request_builder_is_valid(&result)
                && request_builder_error_is_from(&result, conversion_error)
                && request_builder_method(&result) == None
                && request_builder_uri(&result) == None
                && request_builder_version(&result) == None
                && request_builder_headers(&result) == None
                && request_builder_extensions(&result) == None,
        }
    ))]
    pub fn uri<T>(self, uri: T) -> Builder
    where
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: Into<crate::Error>,
    {
        self.and_then(move |mut head| {
            head.uri = match uri.try_into() {
                Ok(uri) => uri,
                Err(error) => return Err(error.into()),
            };
            Ok(head)
        })
    }

    /// Get the URI for this request
    ///
    /// By default this is `/`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let mut req = Request::builder();
    /// assert_eq!(req.uri_ref().unwrap(), "/" );
    ///
    /// req = req.uri("https://www.rust-lang.org/");
    /// assert_eq!(req.uri_ref().unwrap(), "https://www.rust-lang.org/" );
    /// ```
    #[cfg_attr(creusot, ensures(result == request_builder_uri(self)))]
    pub fn uri_ref(&self) -> Option<&Uri> {
        self.inner.as_ref().ok().map(|h| &h.uri)
    }

    /// Set the HTTP version for this request.
    ///
    /// By default this is HTTP/1.1
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let req = Request::builder()
    ///     .version(Version::HTTP_2)
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, ensures(request_builder_error(&result) == request_builder_error(&self)))]
    #[cfg_attr(creusot, ensures(request_builder_version(&result) == if request_builder_is_valid(&self) { Some(&version) } else { None }))]
    #[cfg_attr(creusot, ensures(request_builder_method(&result) == request_builder_method(&self)))]
    #[cfg_attr(creusot, ensures(request_builder_uri(&result) == request_builder_uri(&self)))]
    #[cfg_attr(creusot, ensures(request_builder_headers(&result) == request_builder_headers(&self)))]
    #[cfg_attr(creusot, ensures(request_builder_extensions(&result) == request_builder_extensions(&self)))]
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

    /// Get the HTTP version for this request
    ///
    /// By default this is HTTP/1.1.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let mut req = Request::builder();
    /// assert_eq!(req.version_ref().unwrap(), &Version::HTTP_11 );
    ///
    /// req = req.version(Version::HTTP_2);
    /// assert_eq!(req.version_ref().unwrap(), &Version::HTTP_2 );
    /// ```
    #[cfg_attr(creusot, ensures(result == request_builder_version(self)))]
    pub fn version_ref(&self) -> Option<&Version> {
        self.inner.as_ref().ok().map(|h| &h.version)
    }

    /// Appends a header to this request builder.
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
    /// let req = Request::builder()
    ///     .header("Accept", "text/html")
    ///     .header("X-Custom-Foo", "bar")
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

    /// Get header on this request builder.
    /// when builder has error returns None
    ///
    /// # Example
    ///
    /// ```
    /// # use http::Request;
    /// let req = Request::builder()
    ///     .header("Accept", "text/html")
    ///     .header("X-Custom-Foo", "bar");
    /// let headers = req.headers_ref().unwrap();
    /// assert_eq!( headers["Accept"], "text/html" );
    /// assert_eq!( headers["X-Custom-Foo"], "bar" );
    /// ```
    #[cfg_attr(creusot, ensures(result == request_builder_headers(self)))]
    pub fn headers_ref(&self) -> Option<&HeaderMap<HeaderValue>> {
        self.inner.as_ref().ok().map(|h| &h.headers)
    }

    /// Get headers on this request builder.
    ///
    /// When builder has error returns None.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::{header::HeaderValue, Request};
    /// let mut req = Request::builder();
    /// {
    ///   let headers = req.headers_mut().unwrap();
    ///   headers.insert("Accept", HeaderValue::from_static("text/html"));
    ///   headers.insert("X-Custom-Foo", HeaderValue::from_static("bar"));
    /// }
    /// let headers = req.headers_ref().unwrap();
    /// assert_eq!( headers["Accept"], "text/html" );
    /// assert_eq!( headers["X-Custom-Foo"], "bar" );
    /// ```
    #[cfg_attr(creusot, ensures(match result {
        Some(reference) => Some(&*reference) == request_builder_headers(&*self),
        None => request_builder_headers(&*self) == None,
    }))]
    #[cfg_attr(creusot, ensures(match result {
        Some(reference) => Some(&^reference) == request_builder_headers(&^self),
        None => request_builder_headers(&^self) == None,
    }))]
    #[cfg_attr(creusot, ensures(request_builder_error(&^self) == request_builder_error(&*self)))]
    #[cfg_attr(creusot, ensures(request_builder_method(&^self) == request_builder_method(&*self)))]
    #[cfg_attr(creusot, ensures(request_builder_uri(&^self) == request_builder_uri(&*self)))]
    #[cfg_attr(creusot, ensures(request_builder_version(&^self) == request_builder_version(&*self)))]
    #[cfg_attr(creusot, ensures(request_builder_extensions(&^self) == request_builder_extensions(&*self)))]
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
    /// let req = Request::builder()
    ///     .extension("My Extension")
    ///     .body(())
    ///     .unwrap();
    ///
    /// assert_eq!(req.extensions().get::<&'static str>(),
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

    /// Get a reference to the extensions for this request builder.
    ///
    /// If the builder has an error, this returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::Request;
    /// let req = Request::builder().extension("My Extension").extension(5u32);
    /// let extensions = req.extensions_ref().unwrap();
    /// assert_eq!(extensions.get::<&'static str>(), Some(&"My Extension"));
    /// assert_eq!(extensions.get::<u32>(), Some(&5u32));
    /// ```
    #[cfg_attr(creusot, ensures(result == request_builder_extensions(self)))]
    pub fn extensions_ref(&self) -> Option<&Extensions> {
        self.inner.as_ref().ok().map(|h| &h.extensions)
    }

    /// Get a mutable reference to the extensions for this request builder.
    ///
    /// If the builder has an error, this returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::Request;
    /// let mut req = Request::builder().extension("My Extension");
    /// let mut extensions = req.extensions_mut().unwrap();
    /// assert_eq!(extensions.get::<&'static str>(), Some(&"My Extension"));
    /// extensions.insert(5u32);
    /// assert_eq!(extensions.get::<u32>(), Some(&5u32));
    /// ```
    #[cfg_attr(creusot, ensures(match result {
        Some(reference) => Some(&*reference) == request_builder_extensions(&*self),
        None => request_builder_extensions(&*self) == None,
    }))]
    #[cfg_attr(creusot, ensures(match result {
        Some(reference) => Some(&^reference) == request_builder_extensions(&^self),
        None => request_builder_extensions(&^self) == None,
    }))]
    #[cfg_attr(creusot, ensures(request_builder_error(&^self) == request_builder_error(&*self)))]
    #[cfg_attr(creusot, ensures(request_builder_method(&^self) == request_builder_method(&*self)))]
    #[cfg_attr(creusot, ensures(request_builder_uri(&^self) == request_builder_uri(&*self)))]
    #[cfg_attr(creusot, ensures(request_builder_version(&^self) == request_builder_version(&*self)))]
    #[cfg_attr(creusot, ensures(request_builder_headers(&^self) == request_builder_headers(&*self)))]
    pub fn extensions_mut(&mut self) -> Option<&mut Extensions> {
        match &mut self.inner {
            Ok(head) => Some(&mut head.extensions),
            Err(_) => None,
        }
    }

    /// "Consumes" this builder, using the provided `body` to return a
    /// constructed `Request`.
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
    /// let request = Request::builder()
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, ensures(request_result_head(&result) == request_builder_parts(&self)))]
    #[cfg_attr(creusot, ensures(request_result_body(&result) == if request_builder_is_valid(&self) { Some(&body) } else { None }))]
    #[cfg_attr(creusot, ensures(request_result_error(&result) == request_builder_error(&self)))]
    pub fn body<T>(self, body: T) -> Result<Request<T>> {
        match self.inner {
            Ok(head) => Ok(Request { head, body }),
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
        Err(_) => request_builder_error(&result) == request_builder_error(&self),
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
    #[cfg_attr(creusot, ensures(request_builder_is_valid(&result)))]
    #[cfg_attr(all(creusot, any(not(http_composition_leaf), http_composition_uri_defaults_leaf)), ensures(
        request_builder_has_default_core_fields(&result)
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
        let request = Request::builder().body("some string").unwrap();
        let mapped_request = request.map(|s| {
            assert_eq!(s, "some string");
            123u32
        });
        assert_eq!(mapped_request.body(), &123u32);
    }
}
