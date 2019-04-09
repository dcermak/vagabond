#[derive(Serialize, Deserialize, Debug)]
/// Reply from the Vagrant Cloud API containing the information about a
/// provider.
///
/// See: https://www.vagrantup.com/docs/vagrant-cloud/api.html#providers
pub struct Provider {
    /// name of the provider
    pub name: String,
    /// Is the box for this provider hosted on Vagrant Cloud?
    pub hosted: bool,
    pub hosted_token: Option<String>,
    pub original_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub download_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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
