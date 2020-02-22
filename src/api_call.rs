use has::Has;

use serde::de::DeserializeOwned;
use serde_json;

use crate::{Result, Error};
use crate::parameters::ApiArguments;

/// Quandl API URL used as the base URL for all queries.
///
pub const QUANDL_API_URL: &str = "https://www.quandl.com/api/v3";

/// Trait allowing implementers to submit a request through the Quandl API.
///
/// This trait is implemented by all queries.
///
pub trait ApiCall<T: DeserializeOwned + Clone>: Has<ApiArguments> {
    /// Returns the URL that will be used to submit the query through Quandl's API.
    ///
    fn url(&self) -> String {
        let mut url = QUANDL_API_URL.to_string();

        if let Some(prefix) = self.fmt_prefix() {
            url.push_str(&prefix[..]);
        }

        if let Some(arguments) = self.fmt_arguments() {
            url.push('?');
            url.push_str(&arguments[..]);
        }

        url
    }

    /// Bypass the parsers and retrieve the byte stream received from Quandl directly.
    ///
    fn encoded_data(&self) -> Result<Vec<u8>> {
        crate::download::download(self.url())
    }

    /// Submit a request to the Quandl's API and return a parsed object representing the data
    /// received in a Rust-friendly format.
    ///
    fn send(&self) -> Result<T> {
        let json_data = {
            match String::from_utf8(self.encoded_data()?) {
                Ok(json) => json,
                Err(e) => { return Err(Error::ParsingFailed(e.to_string())); }
            }
        };

        match serde_json::from_str::<T>(&json_data[..]) {
            Ok(data) => Ok(data),
            Err(e) => Err(Error::ParsingFailed(e.to_string())),
        }
    }

    /// If applicable, returns the string that would be appended between the `QUANDL_API_URL` and
    /// the '?' character in a query URL.
    ///
    fn fmt_prefix(&self) -> Option<String> {
        None
    }

    /// If applicable, returns the string that would be appended after the '?' character in a query
    /// URL.
    ///
    fn fmt_arguments(&self) -> Option<String> {
        None
    }
}

impl<'a, T: DeserializeOwned + Clone, A: ApiCall<T>> ApiCall<T> for &'a A {
    fn url(&self) -> String {
        ApiCall::<T>::url(*self)
    }

    fn encoded_data(&self) -> Result<Vec<u8>> {
        ApiCall::<T>::encoded_data(*self)
    }

    fn send(&self) -> Result<T> {
        ApiCall::<T>::send(*self)
    }

    fn fmt_prefix(&self) -> Option<String> {
        ApiCall::<T>::fmt_prefix(*self)
    }

    fn fmt_arguments(&self) -> Option<String> {
        ApiCall::<T>::fmt_arguments(*self)
    }
}

impl<'a, T: DeserializeOwned + Clone, A: ApiCall<T>> ApiCall<T> for &'a mut A {
    fn url(&self) -> String {
        ApiCall::<T>::url(*self)
    }

    fn encoded_data(&self) -> Result<Vec<u8>> {
        ApiCall::<T>::encoded_data(*self)
    }

    fn send(&self) -> Result<T> {
        ApiCall::<T>::send(*self)
    }

    fn fmt_prefix(&self) -> Option<String> {
        ApiCall::<T>::fmt_prefix(*self)
    }

    fn fmt_arguments(&self) -> Option<String> {
        ApiCall::<T>::fmt_arguments(*self)
    }
}
