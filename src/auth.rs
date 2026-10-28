//! Credential handling.

use std::fmt;

use reqwest::header::HeaderValue;

use crate::error::{Error, Result};

/// A validated credential header value that is never printed.
#[derive(Clone)]
pub(crate) struct Secret(HeaderValue);

impl Secret {
    /// Validates a private server key.
    pub(crate) fn server_key(raw: String) -> Result<Self> {
        Self::parse(
            raw,
            "server key must not be empty",
            "server key contains characters that are not allowed in an HTTP header",
        )
    }

    /// Validates a global API key.
    pub(crate) fn global_key(raw: String) -> Result<Self> {
        Self::parse(
            raw,
            "global API key must not be empty",
            "global API key contains characters that are not allowed in an HTTP header",
        )
    }

    fn parse(raw: String, empty: &'static str, invalid: &'static str) -> Result<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(Error::InvalidInput(empty));
        }
        let mut value = HeaderValue::from_str(trimmed).map_err(|_| Error::InvalidInput(invalid))?;
        value.set_sensitive(true);
        Ok(Self(value))
    }

    pub(crate) fn header_value(&self) -> &HeaderValue {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_surrounding_whitespace() {
        let secret = Secret::server_key("  abc123\n".to_owned()).unwrap();
        assert_eq!(secret.header_value().as_bytes(), b"abc123");
    }

    #[test]
    fn rejects_empty_and_blank_keys() {
        assert!(matches!(
            Secret::server_key(String::new()),
            Err(Error::InvalidInput(_))
        ));
        assert!(matches!(
            Secret::server_key(" \t".to_owned()),
            Err(Error::InvalidInput(_))
        ));
    }

    #[test]
    fn rejects_header_unsafe_characters() {
        let err = Secret::server_key("abc\r\ndef".to_owned()).unwrap_err();
        assert!(matches!(err, Error::InvalidInput(_)));
    }

    #[test]
    fn marks_value_sensitive_and_redacts_debug() {
        let secret = Secret::server_key("super-secret".to_owned()).unwrap();
        assert!(secret.header_value().is_sensitive());
        assert_eq!(format!("{secret:?}"), "<redacted>");
    }
}
