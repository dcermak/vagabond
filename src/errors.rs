//! # Error handling module of vagabond
//!
//! This module defines a common error type [`Error`](enum.Error.html) alongside
//! with a [`Result`](type.Result.html) type, which should behave about as
//! you'd expect from a Rust module.
//!
//! A noteworthy convenience method of [`Error`](enum.Error.html) is
//! `into_status()`, which can be used to extract the HTTP status code from a
//! [`Error`](enum.Error.html) (provided it was caused by a HTTP error),
//! resulting in the following (crudely simplified) error handling:
//!
//! ```no_run
//! # extern crate reqwest;
//! # use vagabond::*;
//! # let username = "my_user_name".to_string();
//! # let box_name = "none".to_string();
//! let client = Client::new(None as Option<String>);
//! let vagrant_box = VagrantBox::new(&username, &box_name);
//! let box_res = client.read_box(&vagrant_box);
//! if box_res.is_err() {
//!     box_res.err()
//!         .unwrap()
//!         .into_status()
//!         .and_then(|status| {
//!             println!("oops, got {}, will pretend it was a 404", status);
//!             Some(reqwest::StatusCode::NOT_FOUND)
//!         });
//! }
//! ```

extern crate reqwest;

/// Default Result type as returned by most methods from vagabond
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Fail, Debug)]
/// Common error type for vagabond
pub enum Error {
    #[fail(display = "{}", _0)]
    /// Communication with the API failed due to an external reason
    /// (e.g. API down, no network connection)
    Io(#[fail(cause)] reqwest::Error),

    #[fail(display = "Request failed with status {}: {}", _0, _1)]
    /// The VagrantCloud API reported an error
    ///
    /// The first element of this tuple contains the status code which the API
    /// replied, the second is a semicolon separated list of the human readable
    /// errors reported by the Vagrant Cloud API.
    ApiCallFailure(reqwest::StatusCode, String),

    #[fail(display = "Unexpected response from the API: {}", _0)]
    /// The VagrantCloud API replied with data that couldn't be deserialized
    /// into the expected format
    UnexpectedResponse(String),

    #[fail(display = "Internal error occurred: {}", _0)]
    /// An internal error inside vagabond occurred
    ///
    /// As a API consumer you **really** shouldn't be seeing this kind of
    /// error. If you still do, please report that as a bug.
    InternalError(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Io(err)
    }
}

impl Error {
    /// Extract the status code of this Error if it was caused by an API call
    /// failure, otherwise return None.
    ///
    /// # Examples
    ///
    /// If the error originates from an API call failure:
    /// ```
    /// # use vagabond::errors::*;
    /// let status = reqwest::StatusCode::OK;
    /// let err = Error::ApiCallFailure(status, "error".to_string());
    /// assert_eq!(err.into_status(), Some(status));
    /// ```
    ///
    /// Or if it is another type of error:
    /// ```
    /// # use vagabond::errors::*;
    /// let other_error = Error::InternalError("oops".to_string());
    /// assert_eq!(other_error.into_status(), None);
    /// ```
    pub fn into_status(&self) -> Option<reqwest::StatusCode> {
        match &self {
            Error::ApiCallFailure(st, _) => Some(*st),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
/// Expected payload to be received from Vagrant Cloud on a API call error
struct VagrantCloudErrorPayload {
    /// list of errors that occurred
    /// (this appears to always contain only one element, but don't rely on that)
    errors: Vec<String>,
    /// this should be false, otherwise something is **really** weird
    success: bool,
}

impl From<reqwest::Response> for Error {
    /// Create a [`Error`](enum.Error.html) from a `reqwest::Response`
    fn from(mut resp: reqwest::Response) -> Error {
        let msg: reqwest::Result<VagrantCloudErrorPayload> = resp.json();
        let err_msg: String = match msg {
            Ok(rpl) => rpl.errors.join(", "),
            Err(_) => "".to_string(),
        };
        Error::ApiCallFailure(resp.status(), err_msg)
    }
}
