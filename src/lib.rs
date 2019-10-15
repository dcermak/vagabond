//! # vagabond - a thin wrapper around the Vagrant Cloud API
//!
//! vagabond is a wrapper around the [Vagrant Cloud
//! API](https://www.vagrantup.com/docs/vagrant-cloud/api.html) and can be used
//! to access the service powering https://app.vagrantup.com/ from Rust.
//!
//! All access to the Vagrant Cloud API requires an instance of the
//! [`Client`](struct.Client.html) struct. It can be provided with a API token
//! on construction:
//! ```
//! use vagabond::*;
//! let client = Client::new(Some("my_api_key_here".to_string()));
//! ```
//!
//! The `client` can then be used to perform some actions, e.g. to create a new
//! box:
//! ```no_run
//! # use vagabond::*;
//! # let client = Client::new(Some("my_api_key_here".to_string()));
//! let username = "my_vagrant_cloud_user_name".to_string();
//! let box_name = "awesome_box".to_string();
//! let vagrant_box = VagrantBox::new(&username, &box_name);
//! let res = client.create_box(&vagrant_box);
//! match res {
//!     Ok(b) => println!("Successfully created a box named: {}", b.name),
//!     Err(e) => println!("Oops, got this error: {}", e)
//! };
//! ```
//!
//! ## Nomenclature
//!
//! The Vagrant Cloud API uses the following three terms (and provides API
//! endpoints for each of them):
//! - `box`: refers to a Vagrant box, e.g. "opensuse/openSUSE-Tumbleweed-x86_64".
//! - `version`: a specific version of a Vagrant box, each version can contain
//!   multiple providers, which store the actual virtual machines
//! - `provider`: the actual vagrant box file that will be used to launch the
//!   VM. A `provider` is always tied to a version (which itself is tied to a
//!   `box`).
//!
//! vagabond follows this nomenclature as closely as possible.
//!
//!
//! ## Creating a new Vagrant Box
//!
//! Creating a fresh Vagrant Box that will be not be hosted on Vagrant Cloud,
//! can be achieved as follows (omitting error handling):
//! ```no_run
//! # use vagabond::*;
//! let client = Client::new(Some("my_api_key_here".to_string()));
//!
//! // 1. create a box
//! let username = "my_vagrant_cloud_user_name".to_string();
//! let box_name = "awesome_box".to_string();
//! let vagrant_box = VagrantBox::new(&username, &box_name);
//! client.create_box(&vagrant_box);
//!
//! // 2. create a version
//! let ver = "1.2.3".to_string();
//! let descr = "Release from today!".to_string();
//! let box_version = BoxVersion {
//!     version: &ver,
//!     description: &descr,
//! };
//! client.create_version(&vagrant_box, &box_version);
//!
//! // 3. create a provider
//! let provider_name = "libvirt".to_string();
//! let url = "https://foo.bar.baz/path/to/my/awesome.box".to_string();
//! let provider = BoxProvider {
//!     name: &provider_name,
//!     url: &url,
//! };
//! client.create_provider(&vagrant_box, &box_version, &provider);
//!
//! // 4. release the version
//! client.release_version(&vagrant_box, &box_version);
//! ```
//!
//! ## Logging
//!
//! vagabond uses the [log](https://crates.io/crates/log) crate for logging
//! purposes. API consumers can then use a logging implementation of their
//! choice.
//!
//! vagabond only logs up to the log level `debug` and will **never** log the
//! API token. Note that vagabond uses the
//! [reqwest](https://crates.io/crates/reqwest) crate, which dependencies log
//! extensive amounts of information at the log level `trace`. Using the log
//! level `trace` is therefore **discouraged** as it could leak your API token!

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

use std::fmt;

pub mod api;
pub mod errors;

pub use errors::*;

#[cfg(test)]
mod tests;

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
    /// Create a new Client
    ///
    /// Parameters:
    ///
    /// `token` - optional API token
    ///
    /// Note: some operations will not work without an API token. `vagabond`
    /// will **not** prevent you from performing these, you'll get an Error from
    /// the Vagrant Cloud API instead.
    pub fn new<S>(token: Option<S>) -> Client
    where
        S: Into<String>,
    {
        Client {
            token: token.map(|s| s.into()),
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
                    Err(Error::UnexpectedResponse(response.text()?))
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

    pub fn update_box(&self, vagrant_box: &VagrantBox) -> Result<api::VagrantBox> {
        let url = format!(
            "https://app.vagrantup.com/api/v1/box/{username}/{box_name}",
            username = vagrant_box.username,
            box_name = vagrant_box.name
        );

        let update_box = UpdateBox {
            name: vagrant_box.name,
            short_description: vagrant_box.short_description,
            description: vagrant_box.description,
            is_private: vagrant_box.is_private,
        };

        self.api_call(
            url,
            RequestType::PUT,
            Some(UpdateBoxPayload {
                update_box: &update_box,
            }),
        )
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

    /// this might not work
    pub fn update_version(
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

        let payload = BoxVersion {
            version: box_version.version,
            description: box_version.description,
        };

        self.api_call(url, RequestType::PUT, Some(payload)) as Result<api::Version>
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

    /// Creates a new provider for the given `vagrant_box` and `box_version`.
    ///
    /// Note that the `vagrant_box` and `box_version` already need to exist on
    /// Vagrant Cloud, otherwise the call will fail.
    ///
    /// This function is a wrapper around the following API endpoint: [POST
    /// /api/v1/box/:username/:name/version/:version/providers](https://www.vagrantup.com/docs/vagrant-cloud/api.html#create-a-provider)
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

    pub fn update_provider(
        &self,
        vagrant_box: &VagrantBox,
        box_version: &BoxVersion,
        box_provider: &BoxProvider,
    ) -> Result<api::Provider> {
        let url = format!(
       "https://app.vagrantup.com/api/v1/box/{username}/{box_name}/version/{box_version}/provider/{provider}",
            username = vagrant_box.username,
            box_name = vagrant_box.name,
            box_version = box_version.version,
            provider = box_provider.name
        );

        let prov = Provider {
            provider: box_provider,
        };

        self.api_call(url, RequestType::PUT, Some(prov)) as Result<api::Provider>
    }

    /// Deletes the `box_provider` belonging to the `box_version` of
    /// `vagrant_box`, but does not touch the version or the box itself.
    ///
    /// This function is a wrapper around the [DELETE
    /// /api/v1/box/:username/:name/version/:version/provider/:provider](https://www.vagrantup.com/docs/vagrant-cloud/api.html#delete-a-provider)
    /// API endpoint.
    pub fn delete_provider(
        &self,
        vagrant_box: &VagrantBox,
        box_version: &BoxVersion,
        box_provider: &BoxProvider,
    ) -> Result<api::Provider> {
        let url = format!(
       "https://app.vagrantup.com/api/v1/box/{username}/{box_name}/version/{box_version}/provider/{provider}",
            username = vagrant_box.username,
            box_name = vagrant_box.name,
            box_version = box_version.version,
            provider = box_provider.name
        );

        self.api_call(url, RequestType::DELETE, None as Option<Provider>) as Result<api::Provider>
    }

    /// Creates the provider `box_provider`, belonging to the version
    /// `box_version` of the box `vagrant_box`, creating all required elements
    /// if they should not exist and releasing `box_version`.
    ///
    /// This function is a high level wrapper around the low-level API endpoints
    /// like create_provider, create_box, etc. and can be used to directly
    /// create a usable box on Vagrant Cloud.
    ///
    /// The `delete_other_version` parameter is intended for special purposes
    /// and should be set to false for most cases. This function will delete all
    /// providers with the same provider name in all other versions except in
    /// `box_version`, when `delete_other_version` is set to `true`.
    /// Given the following initial state:
    /// ``` yaml
    /// box:
    ///   versions:
    ///     - version: 1
    ///       providers:
    ///         - name: "libvirt"
    ///           url: "foo"
    ///         - name: "virtualbox"
    ///           url: "bar"
    ///     - version: 2
    ///       providers:
    ///         - name: "virtualbox"
    ///           url: "baz"
    /// ```
    /// calling `ensure_provider_present` with `delete_other_version=true` and
    /// the `libvirt` provider belonging to version `2`, results in the
    /// following:
    /// ``` yaml
    /// box:
    ///   versions:
    ///     - version: 1
    ///       providers:
    ///         - name: "virtualbox"
    ///           url: "bar"
    ///     - version: 2
    ///       providers:
    ///         - name: "libvirt"
    ///           url: "foo_bar"
    ///         - name: "virtualbox"
    ///           url: "baz"
    /// ```
    ///
    /// This is mostly useful if you are building vagrant boxes on a CI that
    /// only keeps the last successful build of each provider.
    ///
    /// This function will also delete versions for which it deleted the last
    /// provider if `delete_other_version=true`.
    pub fn ensure_provider_present(
        &self,
        vagrant_box: &VagrantBox,
        box_version: &BoxVersion,
        box_provider: &BoxProvider,
        delete_other_version: bool,
    ) -> Result<api::VagrantBox> {
        // does this box exist?
        // no => create it and return the result of that operation
        // yes => just return the result
        let box_res = self.read_box(&vagrant_box);
        let box_res = if box_res.is_err() {
            let err = box_res.err().expect("This value must be Err");
            err.into_status()
                .and_then(|st| match st {
                    reqwest::StatusCode::NOT_FOUND => Some(self.create_box(&vagrant_box)),
                    _ => Some(Err(err)),
                })
                .expect("This value must be Some")
        } else {
            Ok(box_res.expect("This value must be Ok"))
        }?;

        // update the box if it some settings aren't matching
        let box_res = if vagrant_box != box_res {
            self.update_box(vagrant_box)?
        } else {
            box_res
        };

        let mut version_present = false;

        // check all versions if their version matches the one we seek to add
        //
        // if the delete_other_version flag is set: delete providers with the
        // same name as box_provider (and cleanup empty versions)
        for ver in box_res.versions.iter() {
            if &ver.version == box_version.version {
                version_present = true;
                continue;
            }
            if delete_other_version {
                match &ver
                    .providers
                    .iter()
                    .find(|prov| &prov.name == box_provider.name)
                {
                    None => (),
                    Some(_) => {
                        let version_to_delete = BoxVersion {
                            version: &ver.version,
                            description: ver
                                .description_markdown
                                .as_ref()
                                .unwrap_or(&box_version.description),
                        };
                        self.delete_provider(vagrant_box, &version_to_delete, box_provider)?;
                        // was that the only provider for this version?
                        // => delete the version too
                        if ver.providers.len() == 1 {
                            self.delete_version(vagrant_box, &version_to_delete)?;
                        }
                    }
                };
            }
        }

        let matching_version = if !version_present {
            self.create_version(vagrant_box, box_version)?
        } else {
            box_res
                .versions
                .into_iter()
                .find(|ver| &ver.version == box_version.version)
                .expect("A matching Version should have been found")
        };

        // redo the same for the provider
        let matching_provider = match matching_version
            .providers
            .into_iter()
            .find(|prov| &prov.name == box_provider.name)
        {
            None => self.create_provider(vagrant_box, box_version, box_provider),
            Some(prov) => Ok(prov) as Result<api::Provider>,
        }?;

        // adjust the provider optionally
        if box_provider != matching_provider {
            self.update_provider(vagrant_box, box_version, box_provider)?;
        }

        self.release_version(vagrant_box, box_version)?;

        self.read_box(vagrant_box)
    }
}

#[derive(Debug, Serialize)]
/// internal struct for sending a BoxProvider via the Vagrant Cloud API
struct Provider<'a, 'b, 'c> {
    provider: &'a BoxProvider<'b, 'c>,
}

#[derive(Debug, Serialize)]
/// internal struct for sending a BoxVersion via the Vagrant Cloud API
struct Version<'a, 'b, 'c> {
    version: &'a BoxVersion<'b, 'c>,
}

#[derive(Debug, Serialize)]
/// internal struct for modifying a Box
struct UpdateBox<'a, 'b, 'c> {
    /// The name of the box.
    name: &'a String,
    /// A short summary of the box.
    short_description: Option<&'b String>,
    /// A longer description of the box. Can be formatted with Markdown.
    description: Option<&'c String>,
    /// (Optional, default: true) - Whether or not this box is private.
    is_private: Option<bool>,
}

#[derive(Debug, Serialize)]
/// payload to send via PUT to update the box
struct UpdateBoxPayload<'a, 'b, 'c, 'd> {
    #[serde(rename = "box")]
    update_box: &'d UpdateBox<'a, 'b, 'c>,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
/// struct representing a provider for a box on Vagrant Cloud
///
/// A BoxProvider represents the downloadable vagrant box for a specific
/// virtualization environment, e.g. virtualbox or libvirt.
pub struct BoxProvider<'a, 'b> {
    /// The name of the provider (e.g. libvirt, virtualbox)
    pub name: &'a String,
    /// A valid URL to download this provider.
    ///
    /// If omitted, you must upload the Vagrant box image for this provider to
    /// Vagrant Cloud before the provider can be used.
    pub url: &'b String,
}

#[derive(Debug, Serialize, PartialEq)]
///
pub struct BoxVersion<'a, 'b> {
    /// The version number of this version.
    pub version: &'a String,
    /// A description for this version. Can be formatted with Markdown.
    pub description: &'b String,
}

#[derive(Debug, Serialize, PartialEq)]
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
            username,
            name: box_name,
            short_description: None,
            description: None,
            is_private: None,
        }
    }
}

/// Compare first with second if second is Some(s), otherwise return false
fn compare_strings(first: &String, second: &Option<String>) -> bool {
    match second {
        Some(val) => first == val,
        None => false,
    }
}

fn cmp_vagrant_providers<'a, 'b>(
    box_provider: &BoxProvider<'a, 'b>,
    api_provider: &api::Provider,
) -> bool {
    (box_provider.name == &api_provider.name)
        && compare_strings(&box_provider.url, &api_provider.original_url)
}

fn cmp_vagrant_versions<'a, 'b>(
    box_version: &BoxVersion<'a, 'b>,
    api_version: &api::Version,
) -> bool {
    (box_version.version == &api_version.version)
        && compare_strings(box_version.description, &api_version.description_markdown)
}

fn cmp_vagrant_boxes<'a, 'b, 'c, 'd>(
    vagrant_box: &VagrantBox<'a, 'b, 'c, 'd>,
    api_vagrant_box: &api::VagrantBox,
) -> bool {
    (vagrant_box.username == &api_vagrant_box.username)
        && (vagrant_box.name == &api_vagrant_box.name)
        && (vagrant_box.short_description == api_vagrant_box.short_description.as_ref())
        && (vagrant_box.description == api_vagrant_box.description_markdown.as_ref())
        && (vagrant_box.is_private == api_vagrant_box.private)
}

impl<'a, 'b> PartialEq<api::Provider> for &BoxProvider<'a, 'b> {
    fn eq(&self, other: &api::Provider) -> bool {
        cmp_vagrant_providers(self, other)
    }
}

impl<'a, 'b> PartialEq<api::Version> for &BoxVersion<'a, 'b> {
    fn eq(&self, other: &api::Version) -> bool {
        cmp_vagrant_versions(self, other)
    }
}

impl<'a, 'b, 'c, 'd> PartialEq<api::VagrantBox> for &VagrantBox<'a, 'b, 'c, 'd> {
    fn eq(&self, other: &api::VagrantBox) -> bool {
        cmp_vagrant_boxes(self, other)
    }
}
