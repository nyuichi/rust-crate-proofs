use std::fmt;

#[cfg(creusot)]
use creusot_std::std::ops::{FnExt as _, FnOnceExt as _};

use super::{ErrorKind, InvalidUri};

#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite, requires, DeepModel, Int, View};

/// The port component of a URI.
pub struct Port<T> {
    port: u16,
    repr: T,
}

impl<T> View for Port<T> {
    type ViewTy = u16;

    #[logic]
    fn view(self) -> Self::ViewTy {
        self.port
    }
}

impl<T> DeepModel for Port<T> {
    type DeepModelTy = Int;

    #[logic]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { self.port@ }
    }
}

impl<T> Port<T> {
    /// Logical observer for the original textual representation. The public
    /// numeric view intentionally omits this value because port equality is
    /// defined only by the parsed number.
    #[cfg(creusot)]
    #[doc(hidden)]
    #[logic]
    pub fn repr_model(&self) -> &T {
        &self.repr
    }

    /// Returns the port number as a `u16`.
    ///
    /// # Examples
    ///
    /// Port as `u16`.
    ///
    /// ```
    /// # use http::uri::Authority;
    /// let authority: Authority = "example.org:80".parse().unwrap();
    ///
    /// let port = authority.port().unwrap();
    /// assert_eq!(port.as_u16(), 80);
    /// ```
    #[ensures(result == self@)]
    pub const fn as_u16(&self) -> u16 {
        self.port
    }
}

impl<T> Port<T>
where
    T: AsRef<str>,
{
    /// Converts a `str` to a port number.
    ///
    /// The supplied `str` must be a valid u16.
    #[requires(<T as AsRef<str>>::as_ref.precondition((&bytes,)))]
    #[ensures(match result {
        Ok(port) => *port.repr_model() == bytes
            && exists<text: &str>
            <T as AsRef<str>>::as_ref.postcondition((&bytes,), text)
                && creusot_std::std::string::parse_u16_model(text@.to_bytes())
                    == Some(port@@),
        Err(error) => error.deep_model() == 3
            && exists<text: &str>
                <T as AsRef<str>>::as_ref.postcondition((&bytes,), text)
                    && creusot_std::std::string::parse_u16_model(text@.to_bytes())
                        == None,
    })]
    pub(crate) fn from_str(bytes: T) -> Result<Self, InvalidUri> {
        match bytes.as_ref().parse::<u16>() {
            Ok(port) => Ok(Port { port, repr: bytes }),
            Err(_) => Err(ErrorKind::InvalidPort.into()),
        }
    }

    /// Returns the port number as a `str`.
    ///
    /// # Examples
    ///
    /// Port as `str`.
    ///
    /// ```
    /// # use http::uri::Authority;
    /// let authority: Authority = "example.org:80".parse().unwrap();
    ///
    /// let port = authority.port().unwrap();
    /// assert_eq!(port.as_str(), "80");
    /// ```
    #[requires(<T as AsRef<str>>::as_ref.precondition((self.repr_model(),)))]
    #[ensures(<T as AsRef<str>>::as_ref.postcondition((self.repr_model(),), result))]
    pub fn as_str(&self) -> &str {
        self.repr.as_ref()
    }
}

impl<T> fmt::Debug for Port<T>
where
    T: fmt::Debug,
{
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            f.deep_model(),
            (^f).deep_model()
        ))
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Port").field(&self.port).finish()
    }
}

impl<T> fmt::Display for Port<T> {
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            f.deep_model(),
            (^f).deep_model()
        ))
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use `u16::fmt` so that it respects any formatting flags that
        // may have been set (like padding, align, etc).
        fmt::Display::fmt(&self.port, f)
    }
}

impl<T> From<Port<T>> for u16 {
    #[ensures(result == port@)]
    fn from(port: Port<T>) -> Self {
        port.as_u16()
    }
}

impl<T> AsRef<str> for Port<T>
where
    T: AsRef<str>,
{
    #[requires(<T as AsRef<str>>::as_ref.precondition((self.repr_model(),)))]
    #[ensures(<T as AsRef<str>>::as_ref.postcondition((self.repr_model(),), result))]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<T, U> PartialEq<Port<U>> for Port<T> {
    #[ensures(result == (self@ == other@))]
    fn eq(&self, other: &Port<U>) -> bool {
        self.port == other.port
    }
}

impl<T> PartialEq<u16> for Port<T> {
    #[ensures(result == (self@ == *other))]
    fn eq(&self, other: &u16) -> bool {
        self.port == *other
    }
}

impl<T> PartialEq<Port<T>> for u16 {
    #[ensures(result == (*self == other@))]
    fn eq(&self, other: &Port<T>) -> bool {
        other.port == *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partialeq_port() {
        let port_a = Port::from_str("8080").unwrap();
        let port_b = Port::from_str("8080").unwrap();
        assert_eq!(port_a, port_b);
    }

    #[test]
    fn partialeq_port_different_reprs() {
        let port_a = Port {
            repr: "8081",
            port: 8081,
        };
        let port_b = Port {
            repr: String::from("8081"),
            port: 8081,
        };
        assert_eq!(port_a, port_b);
        assert_eq!(port_b, port_a);
    }

    #[test]
    fn partialeq_u16() {
        let port = Port::from_str("8080").unwrap();
        // test equals in both directions
        assert_eq!(port, 8080);
        assert_eq!(8080, port);
    }

    #[test]
    fn u16_from_port() {
        let port = Port::from_str("8080").unwrap();
        assert_eq!(8080, u16::from(port));
    }
}
