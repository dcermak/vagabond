//! # API module
//!
//! This module provides structs corresponding to the expected replies from the
//! Vagrant Cloud API.

#[derive(Deserialize, Debug, Default, PartialEq)]
/// Reply from the Vagrant Cloud API containing the information about a
/// provider.
///
/// [Official API
/// documentation](https://www.vagrantup.com/docs/vagrant-cloud/api.html#providers)
pub struct Provider {
    /// Name of the provider
    pub name: String,
    /// Is the box for this provider hosted on Vagrant Cloud?
    pub hosted: bool,
    ///
    pub hosted_token: Option<String>,
    /// Original URL from which the box was downloaded
    pub original_url: Option<String>,
    /// Date string indicating when this box was created
    pub created_at: String,
    /// Date string indicating when this box was last updated
    pub updated_at: String,
    /// Download URL of this box
    pub download_url: String,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
pub struct Version {
    pub version: String,
    pub status: String,
    pub description_html: Option<String>,
    pub description_markdown: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub number: String,
    pub release_url: String,
    pub revoke_url: String,
    pub providers: Vec<Provider>,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
pub struct VagrantBox {
    pub tag: Option<String>,
    pub username: String,
    pub name: String,
    pub private: Option<bool>,
    pub downloads: usize,
    pub created_at: String,
    pub updated_at: String,
    pub short_description: Option<String>,
    pub description_markdown: Option<String>,
    pub description_html: Option<String>,
    pub versions: Vec<Version>,
    pub current_version: Option<Version>,
}

impl<'a, 'b, 'c, 'd> PartialEq<super::VagrantBox<'a, 'b, 'c, 'd>> for &VagrantBox {
    fn eq(&self, other: &super::VagrantBox<'a, 'b, 'c, 'd>) -> bool {
        super::cmp_vagrant_boxes(other, self)
    }
}

impl<'a, 'b> PartialEq<super::BoxVersion<'a, 'b>> for &Version {
    fn eq(&self, other: &super::BoxVersion<'a, 'b>) -> bool {
        super::cmp_vagrant_versions(other, self)
    }
}

impl<'a, 'b> PartialEq<super::BoxProvider<'a, 'b>> for &Provider {
    fn eq(&self, other: &super::BoxProvider<'a, 'b>) -> bool {
        super::cmp_vagrant_providers(other, self)
    }
}
