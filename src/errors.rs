extern crate reqwest;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Fail, Debug)]
/// Common error type for vagabond
pub enum Error {
    #[fail(display = "{}", _0)]
    /// Communication with the API failed due to an external reason
    /// (e.g. API down, no network)
    Io(#[fail(cause)] reqwest::Error),

    #[fail(display = "Request failed with status {}: {}", _0, _1)]
    /// The VagrantCloud API reported an error
    ApiCallFailure(reqwest::StatusCode, String),

    #[fail(display = "Unexpected response from the API: {}", _0)]
    /// The VagrantCloud API replied with data that couldn't be
    /// deserialized into the expected format
    UnexpectedResponse(String),

    #[fail(display = "Internal error occurred: {}", _0)]
    /// An internal error inside vagabond occurred
    InternalError(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Io(err)
    }
}

impl Error {
    /// Extract the status code of this Error if it was caused by an API call
    /// failure, otherwise return None
    pub fn into_status(&self) -> Option<reqwest::StatusCode> {
        match &self {
            Error::ApiCallFailure(st, _) => Some(*st),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct VagrantCloudReply {
    errors: Vec<String>,
    success: bool,
}

impl From<reqwest::Response> for Error {
    fn from(mut resp: reqwest::Response) -> Error {
        let msg: reqwest::Result<VagrantCloudReply> = resp.json();
        let err_msg: String = match msg {
            Ok(rpl) => rpl.errors.join(", "),
            Err(_) => "".to_string(),
        };
        Error::ApiCallFailure(resp.status(), err_msg)
    }
}

// impl From<serde::Error> for Error {
//     fn from(err: serde::Error) -> Error {
//     }
// }
