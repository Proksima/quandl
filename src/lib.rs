//! ## Rust bindings for Quandl v3 API.
//!
//! The goal of this crate is to offer a well documented, complete and easy to use interface to
//! Quandl's RESTful API.
//!
//! This crate uses the `rustc_serialize` crate extensively and thus suffers from some of its
//! limitation. Namely,
//!
//! * When querying for the metadata of a dataset, the field `type` will be missing. This is due to
//!   `type` being a keyword in Rust. Use of this crate assumes knowledge of the layout of the
//!   queried data, so that field was not very important fortunately.
//!
//! * Most public enum's variants have non camel case names to match the naming convention of the
//!   API. The deserializer need the names to match to work properly, thus you will see
//!   `Order::asc` instead of the more readable `Order::Ascending`.
//!
//! Some other design choices of this crate includes
//!
//! * No runtime checking of the query created. This crate makes it as hard as statically possible
//!   to create an invalid query. However, the query will be checked by the Quandl API directly. On
//!   the bright side, we forward Quandl's error messages/codes without pruning any information;
//!   and their error-reporting is very good.
//!
//! * The inclusion of a `batch_query` function that allows users to submit a bunch of query at the
//!   same time. The function returns an iterator which gives the benefit of multithreading
//!   downloads and asynchronicity which are indispensable when doing data mining.
//!
//! * We use the JSON Quandl API for everything but data queries as it often returns more
//!   information. When it comes to the data queries we use the CSV subset of the API as it is
//!   faster and allows to use the `rust-csv` crates which allow you to define your own structs to
//!   receive the data.
//!
//! ### Simple example
//!
//! ```rust
//! extern crate quandl_v3;
//!
//! use quandl_v3::Result;
//! use quandl_v3::prelude::*;
//!
//! fn main() {
//!     let query = {
//!         let mut query = DataQuery::new("WIKI", "AAPL");
//!
//!          query.order(Order::asc)
//!               .end_date(2016, 2, 29)
//!               .start_date(2016, 2, 1)
//!               .column_index(4);
//!
//!          query
//!     };
//!
//!     let response: Vec<(String, f64)> = query.send().unwrap();
//!
//!     // Print the date and closing price for Apple's stock for the month of February 2016.
//!     for data in &response {
//!         println!("{} - {}", data.0, data.1);
//!     }
//! }
//! ```
//!
//! This crate is written in the hope it will be useful. I am in no way affiliated to Quandl and
//! Quandl is not endorsing this crate in any way.
//!
//! Some of the documentation in this crate has been directly copied from Quandl's API
//! Documentation (which is made evident from the links to that documentation directly in this
//! crate's documentation). Those obiously remains the intellectual property of Quandl and were
//! paraphrased to make the use of this crate simpler.
//!
//! [Quandl's Terms of Use](https://www.quandl.com/about/terms)
//!

extern crate zip;
extern crate csv;
extern crate serde;
extern crate reqwest;
extern crate num_cpus;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate has;

mod types;
mod query;
mod api_call;
mod download;
mod parameters;
mod batch_query;

/// This crate's public interface.
///
/// This exclude error handling names to avoid needless conflicts with other crates (e.g.
/// `std::result::Result`).
///
pub mod prelude;

use std::collections::BTreeMap;

/// Crate-wide return type for functions which may fail.
///
pub type Result<T> = ::std::result::Result<T, Error>;

/// Struct for storing a Quandl API error response as-is.
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    /// This field contains more specific information about what went wrong. For example, it could
    /// inform you that a `start_date` is outside a valid range.
    ///
    pub errors: Option<BTreeMap<String, Vec<String>>>,

    /// Hold more generic failure information.
    ///
    pub quandl_error: QuandlError,
}

/// Struct holding Quandl's error code and corresponding message.
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuandlError {
    /// Quandl-specific error code.
    ///
    pub code: String,

    /// Quandl's generic error message matching the above error code.
    ///
    pub message: String,
}

/// Crate-wide error value. This enumerate the only four possible source of failures in this crate.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Is returned when Quandl's reply to a query with an error. The contained `ApiErrorResponse`
    /// contains very verbose information about what went wrong with any specific query.
    ///
    ApiCallFailed(ApiErrorResponse),

    /// Is returned when a problem occurs while exchanging informaiton with the Quandl's servers.
    /// It could mean the Internet connection was lost, that the remote server closed the
    /// connection unexpectedly, etc.
    ///
    /// Unfortunately, the current implementation for network connection (hyper) has very weak
    /// error reporting and thus might leave the user confused as to why such an error is returned.
    ///
    DownloadFailed(String),

    /// Is returned when the received value, assuming Quandl didn't respond with an error and that
    /// there was no download error, breaks one of the parsers' assumption. Most of the time it
    /// would be an error from `rustc_serialize` (which also does not report very meaningful errors
    /// unfortunately) or it could also be a custom message from this library for data which didn't
    /// met the format deserializable by the `rustc_serialize` crate.
    ///
    ParsingFailed(String),

    /// Is returned when an I/O operation fails. This last error is highly system-dependant and
    /// again, the error message string returned are not always very verbose.
    ///
    IoError(String),
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::ApiCallFailed(_)  => "Quandl's server responded with an error.",
            &Error::DownloadFailed(_) => "Download failed.",
            &Error::ParsingFailed(_)  => "Parsing data failed.",
            &Error::IoError(_)        => "Underlying system I/O error.",
        }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            &Error::ApiCallFailed(ref e) => {
                if e.errors.is_some() && !e.errors.as_ref().unwrap().is_empty() {
                    let (object, what) = e.errors.as_ref().unwrap().iter().next().unwrap();

                    write!(f, "{}", {
                        what.iter().fold(format!("{} - ", object), |xs, x| format!("{} {}", xs, x))
                    })
                } else {
                    write!(f, "{}", e.quandl_error.message)
                }
            },

            &Error::DownloadFailed(ref s) => {
                write!(f, "download failed with error '{}'.", s)
            },

            &Error::ParsingFailed(ref s) => {
                write!(f, "parsing encoded data failed with error '{}'.", s)
            },

            &Error::IoError(ref s) => {
                write!(f, "I/O operation failed with error '{}'.", s)
            },
        }
    }
}
