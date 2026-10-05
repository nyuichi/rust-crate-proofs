use std::convert::TryInto;

use super::{Authority, Parts, PathAndQuery, Scheme};
use crate::Uri;

#[cfg(creusot)]
#[allow(unused_imports)]
use creusot_std::std::ops::{FnExt as _, FnOnceExt as _};
#[cfg(creusot)]
#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite, requires, DeepModel};

/// Whether the URI builder currently holds parts instead of a conversion
/// error.
#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn uri_builder_is_valid(builder: &Builder) -> bool {
    match &builder.parts {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// The exact parts currently held by a valid URI builder.
#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn uri_builder_parts<'a>(builder: &'a Builder) -> Option<&'a Parts> {
    match &builder.parts {
        Ok(parts) => Some(parts),
        Err(_) => None,
    }
}

/// The exact stored error, including its payload, when a URI builder is in
/// the error state.
#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn uri_builder_error<'a>(builder: &'a Builder) -> Option<crate::ErrorModelRef<'a>> {
    match &builder.parts {
        Ok(_) => None,
        Err(error) => Some(crate::error_model_ref(error)),
    }
}

/// Whether a stored builder error is the result of converting `source` into
/// the shared HTTP error type.
#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn uri_builder_error_is_from<E>(builder: &Builder, source: E) -> bool
where
    E: Into<crate::Error>,
{
    match &builder.parts {
        Ok(_) => false,
        Err(error) => <E as Into<crate::Error>>::into.postcondition((source,), *error),
    }
}

/// The current scheme field when a URI builder is valid.
#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn uri_builder_scheme<'a>(builder: &'a Builder) -> Option<&'a Scheme> {
    match uri_builder_parts(builder) {
        Some(parts) => match &parts.scheme {
            Some(scheme) => Some(scheme),
            None => None,
        },
        None => None,
    }
}

/// The current authority field when a URI builder is valid.
#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn uri_builder_authority<'a>(builder: &'a Builder) -> Option<&'a Authority> {
    match uri_builder_parts(builder) {
        Some(parts) => match &parts.authority {
            Some(authority) => Some(authority),
            None => None,
        },
        None => None,
    }
}

/// The current path-and-query field when a URI builder is valid.
#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn uri_builder_path_and_query<'a>(builder: &'a Builder) -> Option<&'a PathAndQuery> {
    match uri_builder_parts(builder) {
        Some(parts) => match &parts.path_and_query {
            Some(path_and_query) => Some(path_and_query),
            None => None,
        },
        None => None,
    }
}

/// Whether a valid URI builder has all three fields unset.
#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn uri_builder_has_default_parts(builder: &Builder) -> bool {
    pearlite! {
        match uri_builder_parts(builder) {
            Some(parts) => {
                parts@.scheme == None
                    && parts@.authority == None
                    && parts@.path_and_query == None
            }
            None => false,
        }
    }
}

/// Whether a valid URI builder contains the empty path-and-query value.
#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn uri_builder_has_empty_path_and_query(builder: &Builder) -> bool {
    pearlite! {
        match uri_builder_path_and_query(builder) {
            Some(path_and_query) => {
                path_and_query@.bytes.len() == 0 && path_and_query@.query == None
            }
            None => false,
        }
    }
}

/// The result relation for `Builder::build`, preserving a stored conversion
/// error or the component and error model returned by `Uri::from_parts`.
#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn uri_builder_build_post(
    builder: &Builder,
    outcome: &std::result::Result<Uri, crate::Error>,
) -> bool {
    pearlite! {
        match (&builder.parts, outcome) {
            (Err(_), Ok(_)) => false,
            (Err(_), Err(error)) => {
                uri_builder_error(builder) == Some(crate::error_model_ref(error))
            }
            (Ok(parts), Ok(uri)) => {
                super::uri_parts_model_is_valid(parts@)
                    && uri@.scheme == super::uri_model_from_parts(parts@).scheme
                    && uri@.authority == super::uri_model_from_parts(parts@).authority
                    && uri@.path_and_query == super::uri_model_from_parts(parts@).path_and_query
            }
            (Ok(parts), Err(error)) => {
                exists<parts_error: super::InvalidUriParts>
                    <super::InvalidUriParts as Into<crate::Error>>::into
                        .postcondition((parts_error,), *error)
                        && super::uri_parts_error_matches(parts@, parts_error.deep_model())
            }
        }
    }
}

/// A builder for `Uri`s.
///
/// This type can be used to construct an instance of `Uri`
/// through a builder pattern.
#[cfg_attr(not(http_uri_builder_leaf), derive(Debug))]
pub struct Builder {
    parts: Result<Parts, crate::Error>,
}

impl Builder {
    /// Creates a new default instance of `Builder` to construct a `Uri`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let uri = uri::Builder::new()
    ///     .scheme("https")
    ///     .authority("hyper.rs")
    ///     .path_and_query("/")
    ///     .build()
    ///     .unwrap();
    /// ```
    #[inline]
    #[cfg_attr(creusot, ensures(uri_builder_is_valid(&result)))]
    #[cfg_attr(creusot, ensures(uri_builder_has_default_parts(&result)))]
    pub fn new() -> Builder {
        Builder::default()
    }

    /// Set the `Scheme` for this URI.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let mut builder = uri::Builder::new();
    /// builder.scheme("https");
    /// ```
    #[cfg_attr(creusot, requires(!uri_builder_is_valid(&self) || (
        <T as TryInto<Scheme>>::try_into.precondition((scheme,))
        && forall<conversion_error: <T as TryInto<Scheme>>::Error>
            <T as TryInto<Scheme>>::try_into.postcondition((scheme,), Err(conversion_error))
                ==> <<T as TryInto<Scheme>>::Error as Into<crate::Error>>::into
                    .precondition((conversion_error,))
    )))]
    #[cfg_attr(creusot, ensures(!uri_builder_is_valid(&self) ==>
        uri_builder_error(&result) == uri_builder_error(&self)))]
    #[cfg_attr(creusot, ensures(uri_builder_is_valid(&self) ==>
        exists<converted: std::result::Result<Scheme, <T as TryInto<Scheme>>::Error>>
            <T as TryInto<Scheme>>::try_into.postcondition((scheme,), converted)
                && match converted {
                    Ok(new_scheme) => uri_builder_is_valid(&result)
                        && uri_builder_scheme(&result) == Some(&new_scheme)
                        && uri_builder_error(&result) == None
                        && uri_builder_authority(&result) == uri_builder_authority(&self)
                        && uri_builder_path_and_query(&result)
                            == uri_builder_path_and_query(&self),
                    Err(conversion_error) => !uri_builder_is_valid(&result)
                        && uri_builder_error_is_from(&result, conversion_error)
                        && uri_builder_scheme(&result) == None
                        && uri_builder_authority(&result) == None
                        && uri_builder_path_and_query(&result) == None,
                }
    ))]
    pub fn scheme<T>(self, scheme: T) -> Self
    where
        T: TryInto<Scheme>,
        <T as TryInto<Scheme>>::Error: Into<crate::Error>,
    {
        self.map(move |mut parts| {
            let scheme = match scheme.try_into() {
                Ok(scheme) => scheme,
                Err(error) => return Err(error.into()),
            };
            parts.scheme = Some(scheme);
            Ok(parts)
        })
    }

    /// Set the `Authority` for this URI.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let uri = uri::Builder::new()
    ///     .authority("tokio.rs")
    ///     .build()
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, requires(!uri_builder_is_valid(&self) || (
        <T as TryInto<Authority>>::try_into.precondition((auth,))
        && forall<conversion_error: <T as TryInto<Authority>>::Error>
            <T as TryInto<Authority>>::try_into.postcondition((auth,), Err(conversion_error))
                ==> <<T as TryInto<Authority>>::Error as Into<crate::Error>>::into
                    .precondition((conversion_error,))
    )))]
    #[cfg_attr(creusot, ensures(!uri_builder_is_valid(&self) ==>
        uri_builder_error(&result) == uri_builder_error(&self)))]
    #[cfg_attr(creusot, ensures(uri_builder_is_valid(&self) ==>
        exists<converted: std::result::Result<Authority, <T as TryInto<Authority>>::Error>>
            <T as TryInto<Authority>>::try_into.postcondition((auth,), converted)
                && match converted {
                    Ok(new_authority) => uri_builder_is_valid(&result)
                        && uri_builder_authority(&result) == Some(&new_authority)
                        && uri_builder_error(&result) == None
                        && uri_builder_scheme(&result) == uri_builder_scheme(&self)
                        && uri_builder_path_and_query(&result)
                            == uri_builder_path_and_query(&self),
                    Err(conversion_error) => !uri_builder_is_valid(&result)
                        && uri_builder_error_is_from(&result, conversion_error)
                        && uri_builder_scheme(&result) == None
                        && uri_builder_authority(&result) == None
                        && uri_builder_path_and_query(&result) == None,
                }
    ))]
    pub fn authority<T>(self, auth: T) -> Self
    where
        T: TryInto<Authority>,
        <T as TryInto<Authority>>::Error: Into<crate::Error>,
    {
        self.map(move |mut parts| {
            let auth = match auth.try_into() {
                Ok(auth) => auth,
                Err(error) => return Err(error.into()),
            };
            parts.authority = Some(auth);
            Ok(parts)
        })
    }

    /// Set the `PathAndQuery` for this URI.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let uri = uri::Builder::new()
    ///     .path_and_query("/hello?foo=bar")
    ///     .build()
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, requires(!uri_builder_is_valid(&self) || (
        <T as TryInto<PathAndQuery>>::try_into.precondition((p_and_q,))
        && forall<conversion_error: <T as TryInto<PathAndQuery>>::Error>
            <T as TryInto<PathAndQuery>>::try_into.postcondition((p_and_q,), Err(conversion_error))
                ==> <<T as TryInto<PathAndQuery>>::Error as Into<crate::Error>>::into
                    .precondition((conversion_error,))
    )))]
    #[cfg_attr(creusot, ensures(!uri_builder_is_valid(&self) ==>
        uri_builder_error(&result) == uri_builder_error(&self)))]
    #[cfg_attr(all(creusot, http_uri_builder_leaf, not(http_composition_leaf)), ensures(uri_builder_is_valid(&self) ==>
        exists<converted: std::result::Result<PathAndQuery, <T as TryInto<PathAndQuery>>::Error>>
            <T as TryInto<PathAndQuery>>::try_into.postcondition((p_and_q,), converted)
                && match converted {
                    Ok(new_path_and_query) => uri_builder_is_valid(&result)
                        && uri_builder_path_and_query(&result) == Some(&new_path_and_query)
                        && uri_builder_error(&result) == None
                        && uri_builder_scheme(&result) == uri_builder_scheme(&self)
                        && uri_builder_authority(&result) == uri_builder_authority(&self),
                    Err(conversion_error) => exists<converted_error: crate::Error>
                        <<T as TryInto<PathAndQuery>>::Error as Into<crate::Error>>::into
                            .postcondition((conversion_error,), converted_error)
                            && if crate::error_is_empty_uri(&converted_error) {
                                uri_builder_is_valid(&result)
                                    && uri_builder_has_empty_path_and_query(&result)
                                    && uri_builder_error(&result) == None
                                    && uri_builder_scheme(&result) == uri_builder_scheme(&self)
                                    && uri_builder_authority(&result) == uri_builder_authority(&self)
                            } else {
                                !uri_builder_is_valid(&result)
                                    && uri_builder_error_is_from(&result, conversion_error)
                                    && uri_builder_error(&result)
                                        == Some(crate::error_model_ref(&converted_error))
                                    && uri_builder_scheme(&result) == None
                                    && uri_builder_authority(&result) == None
                                    && uri_builder_path_and_query(&result) == None
                            },
                }
    ))]
    pub fn path_and_query<T>(self, p_and_q: T) -> Self
    where
        T: TryInto<PathAndQuery>,
        <T as TryInto<PathAndQuery>>::Error: Into<crate::Error>,
    {
        self.map(move |mut parts| {
            let p_and_q = match p_and_q.try_into() {
                Ok(p_and_q) => p_and_q,
                Err(err) => {
                    let err = err.into();
                    if err.is_empty_uri() {
                        PathAndQuery::empty()
                    } else {
                        return Err(err);
                    }
                }
            };
            parts.path_and_query = Some(p_and_q);
            Ok(parts)
        })
    }

    /// Consumes this builder, and tries to construct a valid `Uri` from
    /// the configured pieces.
    ///
    /// # Errors
    ///
    /// This function may return an error if any previously configured argument
    /// failed to parse or get converted to the internal representation. For
    /// example if an invalid `scheme` was specified via `scheme("!@#%/^")`
    /// the error will be returned when this function is called rather than
    /// when `scheme` was called.
    ///
    /// Additionally, the various forms of URI require certain combinations of
    /// parts to be set to be valid. If the parts don't fit into any of the
    /// valid forms of URI, a new error is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::*;
    ///
    /// let uri = Uri::builder()
    ///     .build()
    ///     .unwrap();
    /// ```
    #[cfg_attr(creusot, ensures(uri_builder_build_post(&self, &result)))]
    pub fn build(self) -> Result<Uri, crate::Error> {
        let parts = match self.parts {
            Ok(parts) => parts,
            Err(error) => return Err(error),
        };
        match Uri::from_parts(parts) {
            Ok(uri) => Ok(uri),
            Err(error) => Err(error.into()),
        }
    }

    // private

    #[cfg_attr(creusot, requires(match self.parts {
        Ok(parts) => func.precondition((parts,)),
        Err(_) => true,
    }))]
    #[cfg_attr(creusot, ensures(match self.parts {
        Ok(parts) => func.postcondition_once((parts,), result.parts),
        Err(_) => uri_builder_error(&result) == uri_builder_error(&self),
    }))]
    fn map<F>(self, func: F) -> Self
    where
        F: FnOnce(Parts) -> Result<Parts, crate::Error>,
    {
        let parts = match self.parts {
            Ok(parts) => func(parts),
            Err(error) => Err(error),
        };
        Builder {
            parts,
        }
    }
}

impl Default for Builder {
    #[inline]
    #[cfg_attr(creusot, ensures(uri_builder_is_valid(&result)))]
    #[cfg_attr(creusot, ensures(uri_builder_has_default_parts(&result)))]
    fn default() -> Builder {
        Builder {
            parts: Ok(Parts::default()),
        }
    }
}

impl From<Uri> for Builder {
    #[cfg_attr(creusot, ensures(uri_builder_is_valid(&result)))]
    #[cfg_attr(creusot, ensures(match uri_builder_parts(&result) {
        Some(parts) => parts@.scheme == super::uri_parts_model(uri@).scheme
            && parts@.authority == super::uri_parts_model(uri@).authority
            && parts@.path_and_query == super::uri_parts_model(uri@).path_and_query,
        None => false,
    }))]
    #[cfg_attr(creusot, ensures(uri_builder_error(&result) == None))]
    fn from(uri: Uri) -> Self {
        Self {
            parts: Ok(uri.into_parts()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_from_str() {
        let uri = Builder::new()
            .scheme(Scheme::HTTP)
            .authority("hyper.rs")
            .path_and_query("/foo?a=1")
            .build()
            .unwrap();
        assert_eq!(uri.scheme_str(), Some("http"));
        assert_eq!(uri.authority().unwrap().host(), "hyper.rs");
        assert_eq!(uri.path(), "/foo");
        assert_eq!(uri.query(), Some("a=1"));
    }

    #[test]
    fn build_from_string() {
        for i in 1..10 {
            let uri = Builder::new()
                .path_and_query(format!("/foo?a={}", i))
                .build()
                .unwrap();
            let expected_query = format!("a={}", i);
            assert_eq!(uri.path(), "/foo");
            assert_eq!(uri.query(), Some(expected_query.as_str()));
        }
    }

    #[test]
    fn build_from_string_ref() {
        for i in 1..10 {
            let p_a_q = format!("/foo?a={}", i);
            let uri = Builder::new().path_and_query(&p_a_q).build().unwrap();
            let expected_query = format!("a={}", i);
            assert_eq!(uri.path(), "/foo");
            assert_eq!(uri.query(), Some(expected_query.as_str()));
        }
    }

    #[test]
    fn build_from_empty_path_and_query() {
        let uri = Builder::new()
            .scheme(Scheme::HTTP)
            .authority("localhost:8080")
            .path_and_query("")
            .build()
            .unwrap();

        assert_eq!(uri, "http://localhost:8080");
        assert_eq!(uri.path(), "/");
    }

    #[test]
    fn empty_path_and_query_remains_strict() {
        assert!(PathAndQuery::try_from("").is_err());
    }

    #[test]
    fn authority_form_path_and_query_remains_strict() {
        assert!(Builder::new()
            .path_and_query("localhost:8080")
            .build()
            .is_err());
    }

    #[test]
    fn build_from_uri() {
        let original_uri = Uri::default();
        let uri = Builder::from(original_uri.clone()).build().unwrap();
        assert_eq!(original_uri, uri);
    }

    #[test]
    fn build_star_for_http2() {
        let uri = Builder::new()
            .scheme("https")
            .authority("example.com")
            .path_and_query("*")
            .build()
            .unwrap();

        assert_eq!(uri.scheme(), Some(&Scheme::HTTPS));
        assert_eq!(uri.host(), Some("example.com"));
        assert_eq!(uri.path(), "*");
    }
}
