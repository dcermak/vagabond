extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

use std::fmt;

pub mod api;
pub mod errors;

pub use errors::*;

#[derive(Debug)]
/// Available HTTP request types
enum RequestType {
    GET,
    POST,
    DELETE,
    PUT,
}

impl fmt::Display for RequestType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                RequestType::GET => "GET",
                RequestType::POST => "POST",
                RequestType::DELETE => "DELETE",
                RequestType::PUT => "PUT",
            }
        )
    }
}

#[derive(Debug)]
/// Client for communication with the Vagrant Cloud API
pub struct Client {
    token: Option<String>,
}

impl Client {
    pub fn new<S>(token: Option<S>) -> Client
    where
        S: Into<String>,
    {
        Client {
            token: match token {
                Some(s) => Some(s.into()),
                None => None,
            },
        }
    }

    /// General purpose method to perform a call to the Vagrant Cloud API
    ///
    /// Parameters:
    /// - `api_url`: URL to which the call will be made. Must be convertible to
    ///     a `String`. If it cannot be converted to a valid reqwest::Url, then
    ///     this function returns a `Error::IntenralError`.
    /// - `request_type`: type of HTTP request to be performed
    /// - `payload`: Optional payload, will be send as serialized as json with
    ///     the request (must thus support the Deserialize trait from serde)
    ///
    /// This function performs a call to the specified `api_url` with the
    /// specified `request_type`.
    /// If the client contains a `token`, then it is passed along as the header
    /// "Authorization: Bearer {token}".
    /// If the payload is `Some(p)`, then the `p` is serialized to json and send
    /// along with the request.
    ///
    /// The call to the API is considered successful, if one of the following
    /// HTTP status codes is returned:
    /// - 200 OK
    /// - 201 Created
    /// - 204 No Content
    ///
    /// Then received data are deserialized from json into a new instance of
    /// type `R`.
    ///
    /// Returns:
    /// - Result<R>: where R is some type that can be deserialized:
    ///     * Ok(res): res the received reply from the API deserialized from JSON
    ///     * Err(e): any errors that occurred during the call are returned
    fn api_call<S, R, P>(
        &self,
        api_url: S,
        request_type: RequestType,
        payload: Option<P>,
    ) -> Result<R>
    where
        for<'de> R: serde::Deserialize<'de>,
        S: Into<String>,
        P: serde::Serialize,
    {
        let client = reqwest::Client::new();

        let url = match reqwest::Url::parse(&api_url.into()) {
            Ok(u) => u,
            Err(e) => {
                return Err(Error::InternalError(format!(
                    "error parsing the url, got: '{}'",
                    e
                )));
            }
        };

        debug!("Performing a {} request to {}", request_type, url);

        let mut builder = match request_type {
            RequestType::GET => client.get(url),
            RequestType::POST => client.post(url),
            RequestType::DELETE => client.delete(url),
            RequestType::PUT => client.put(url),
        };
        builder = match &self.token {
            Some(t) => {
                debug!("Passing Authorization token");
                builder.header("Authorization", format!("Bearer {}", t))
            }
            _ => builder,
        };
        builder = match payload {
            Some(p) => {
                debug!(
                    "Sending the following payload: {}",
                    serde_json::to_string(&p)
                        .or(Ok("Error serializing payload".to_string())
                            as std::result::Result<String, serde_json::Error>)
                        .unwrap()
                );
                builder.json(&p)
            }
            _ => builder,
        };

        let mut response = builder.send()?;

        debug!("Received status {}", response.status());
        match response.status() {
            reqwest::StatusCode::OK
            | reqwest::StatusCode::CREATED
            | reqwest::StatusCode::NO_CONTENT => match response.json() {
                Ok(r) => Ok(r),
                Err(e) => {
                    debug!("Received unexpected response: {:?}", e);
                    Err(Error::UnexpectedResponse(response.text()?.into()))
                }
            },
            _ => Err(response)?,
        }
    }

    pub fn create_box(&self, vagrant_box: &VagrantBox) -> Result<api::VagrantBox> {
        let url = "https://app.vagrantup.com/api/v1/boxes/";

        self.api_call(url, RequestType::POST, Some(vagrant_box)) as Result<api::VagrantBox>
    }

    pub fn delete_box(&self, vagrant_box: &VagrantBox) -> Result<api::VagrantBox> {
        let url = format!(
            "https://app.vagrantup.com/api/v1/box/{}/{}",
            vagrant_box.username, vagrant_box.name
        );

        self.api_call(url, RequestType::DELETE, None as Option<VagrantBox>)
            as Result<api::VagrantBox>
    }

    pub fn read_box(&self, vagrant_box: &VagrantBox) -> Result<api::VagrantBox> {
        let url = format!(
            "https://app.vagrantup.com/api/v1/box/{}/{}",
            vagrant_box.username, vagrant_box.name
        );

        self.api_call(url, RequestType::GET, None as Option<VagrantBox>) as Result<api::VagrantBox>
    }

    pub fn create_version(
        &self,
        vagrant_box: &VagrantBox,
        box_version: &BoxVersion,
    ) -> Result<api::Version> {
        let url = format!(
            "https://app.vagrantup.com/api/v1/box/{}/{}/versions",
            vagrant_box.username, vagrant_box.name
        );

        let ver: Version = Version {
            version: box_version,
        };

        self.api_call(url, RequestType::POST, Some(ver)) as Result<api::Version>
    }

    pub fn read_version(
        &self,
        vagrant_box: &VagrantBox,
        box_version: &BoxVersion,
    ) -> Result<api::Version> {
        let url = format!(
            "https://app.vagrantup.com/api/v1/box/{username}/{box_name}/version/{box_version}",
            username = vagrant_box.username,
            box_name = vagrant_box.name,
            box_version = box_version.version
        );
        self.api_call(url, RequestType::GET, None as Option<Version>) as Result<api::Version>
    }

    pub fn delete_version(
        &self,
        vagrant_box: &VagrantBox,
        box_version: &BoxVersion,
    ) -> Result<api::Version> {
        let url = format!(
            "https://app.vagrantup.com/api/v1/box/{username}/{box_name}/version/{box_version}",
            username = vagrant_box.username,
            box_name = vagrant_box.name,
            box_version = box_version.version
        );

        self.api_call(url, RequestType::DELETE, None as Option<Version>) as Result<api::Version>
    }

    pub fn release_version(
        &self,
        vagrant_box: &VagrantBox,
        box_version: &BoxVersion,
    ) -> Result<api::Version> {
        let url = format!(
            "https://app.vagrantup.com/api/v1/box/{username}/{name}/version/{box_version}/release",
            username = vagrant_box.username,
            name = vagrant_box.name,
            box_version = box_version.version
        );

        self.api_call(url, RequestType::PUT, None as Option<Version>) as Result<api::Version>
    }

    pub fn create_provider(
        &self,
        vagrant_box: &VagrantBox,
        box_version: &BoxVersion,
        box_provider: &BoxProvider,
    ) -> Result<api::Provider> {
        let url = format!(
            "https://app.vagrantup.com/api/v1/box/{username}/{box_name}/version/{box_version}/providers",
            username = vagrant_box.username,
            box_name = vagrant_box.name,
            box_version = box_version.version
        );

        let prov = Provider {
            provider: box_provider,
        };

        self.api_call(url, RequestType::POST, Some(prov)) as Result<api::Provider>
    }
}

#[derive(Debug, Serialize)]
struct Provider<'a, 'b, 'c> {
    provider: &'a BoxProvider<'b, 'c>,
}

#[derive(Debug, Serialize)]
struct Version<'a, 'b, 'c> {
    version: &'a BoxVersion<'b, 'c>,
}

#[derive(Debug, Serialize)]
pub struct BoxProvider<'a, 'b> {
    /// The name of the provider
    pub name: &'a String,
    /// A valid URL to download this provider.
    ///
    /// If omitted, you must upload the Vagrant box image for this provider to
    /// Vagrant Cloud before the provider can be used.
    pub url: &'b String,
}

#[derive(Debug, Serialize)]
///
pub struct BoxVersion<'a, 'b> {
    /// The version number of this version.
    pub version: &'a String,
    /// A description for this version. Can be formatted with Markdown.
    pub description: &'b String,
}

#[derive(Debug, Serialize)]
pub struct VagrantBox<'a, 'b, 'c, 'd> {
    /// The username of the organization that will own this box
    pub username: &'a String,
    /// The name of the box
    pub name: &'b String,
    /// A short summary of the box
    pub short_description: Option<&'c String>,
    /// A longer description of the box. Can be formatted with Markdown.
    pub description: Option<&'d String>,
    /// Whether or not this box is private.
    pub is_private: Option<bool>,
}

impl<'a, 'b, 'c, 'd> VagrantBox<'a, 'b, 'c, 'd> {
    pub fn new(username: &'a String, box_name: &'b String) -> VagrantBox<'a, 'b, 'c, 'd> {
        VagrantBox {
            username: username,
            name: box_name,
            short_description: None,
            description: None,
            is_private: None,
        }
    }
}