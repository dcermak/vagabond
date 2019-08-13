extern crate rand;
extern crate stderrlog;

extern crate vagabond;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

use std::env;
use stderrlog::*;

use rand::distributions::{Distribution, Standard};

const RANDOM_BOXNAME_POSTFIX_LENGTH: usize = 5;

struct TestFixture {
    client: vagabond::Client,
    user: String,
    box_name: String,
}

impl TestFixture {
    fn new(box_name: Option<&str>) -> TestFixture {
        let _ = stderrlog::new()
            .module("vagabond")
            .module(module_path!())
            .verbosity(4)
            .timestamp(Timestamp::Millisecond)
            .init();

        let rng = rand::thread_rng();
        let postfix: String = Standard
            .sample_iter(rng)
            .filter(|v: &char| v.is_ascii_alphabetic() || v.is_ascii_alphanumeric())
            .take(RANDOM_BOXNAME_POSTFIX_LENGTH)
            .collect::<String>();

        let fixture = TestFixture {
            client: vagabond::Client::new(Some(env::var("ATLAS_TOKEN").unwrap())),
            user: env::var("ATLAS_USER").unwrap(),
            // append a random ASCII string to the boxname, so that we can run
            // the tests concurrently
            box_name: box_name.map_or("test_box".to_string(), |b| b.to_string()) + &postfix,
        };
        debug!(
            "Deleting previously existing box: {:?}",
            fixture.client.delete_box(&fixture.get_vagrant_box())
        );

        fixture
    }

    fn get_vagrant_box(&self) -> vagabond::VagrantBox {
        vagabond::VagrantBox::new(&self.user, &self.box_name)
    }

    fn box_create(&self) -> vagabond::Result<vagabond::api::VagrantBox> {
        self.client.create_box(&self.get_vagrant_box())
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        debug!(
            "Deleting Box: {:?}",
            self.client.delete_box(&self.get_vagrant_box())
        );
    }
}

struct VersionFixture {
    test_fixture: TestFixture,
    version: String,
    description: String,
}

impl VersionFixture {
    fn new(
        box_name: Option<&str>,
        version: Option<&str>,
        description: Option<&str>,
    ) -> VersionFixture {
        let test_fixture = TestFixture::new(box_name);
        test_fixture.box_create().unwrap();
        VersionFixture {
            test_fixture: test_fixture,
            version: version.map_or("1.2.3".to_string(), |v| v.to_string()),
            description: description.map_or("This is a test Box".to_string(), |d| d.to_string()),
        }
    }

    fn get_vagrant_version(&self) -> vagabond::BoxVersion {
        vagabond::BoxVersion {
            version: &self.version,
            description: &self.description,
        }
    }

    fn version_create(&self) -> vagabond::Result<vagabond::api::Version> {
        self.test_fixture.client.create_version(
            &self.test_fixture.get_vagrant_box(),
            &self.get_vagrant_version(),
        )
    }
}

#[test]
fn box_creation_should_succeed() {
    let fixture = TestFixture::new(None);

    let box_create_res = fixture.box_create();

    assert!(box_create_res.is_ok());

    let box_res = box_create_res.unwrap();
    assert_eq!(box_res.username, fixture.user);
    assert_eq!(box_res.name, fixture.box_name);
}

#[test]
fn version_creation_should_succeed() {
    let version = "2.1.3";
    let description = "This is a box for version testing";
    let fixture = VersionFixture::new(Some("test_version_box"), Some(version), Some(description));

    let ver_create_res = fixture.version_create();
    assert!(ver_create_res.is_ok());

    let version_result = ver_create_res.unwrap();

    assert_eq!(version_result.version, version);
    if version_result.description_markdown.is_some() {
        assert_eq!(version_result.description_markdown.unwrap(), description);
    }
}

#[test]
fn version_deletion_should_work() {
    let version = "42.21.11";
    let fixture = VersionFixture::new(Some("test_version_box"), Some(version), None);

    fixture.version_create().unwrap();

    let delete_res = fixture.test_fixture.client.delete_version(
        &fixture.test_fixture.get_vagrant_box(),
        &fixture.get_vagrant_version(),
    );

    assert!(delete_res.is_ok());

    assert_eq!(delete_res.unwrap().version, fixture.version);
}

lazy_static! {
    static ref VERSION: String = "15.16.17".to_string();
    static ref VERSION2: String = "31.29.1".to_string();
    static ref VERSION3: String = "28.1".to_string();
    static ref VERSION4: String = "29".to_string();
    static ref BOX_NAME: String = "fresh_box".to_string();
    static ref VER_DESCR: String = "version 15!!".to_string();
    static ref LIBVIRT: String = "libvirt".to_string();
    static ref VIRTUALBOX: String = "virtualbox".to_string();
    static ref URL: String = "https://foo.bar.baz/my/box/15.16.17/img.box".to_string();
    static ref URL2: String = "https://foo.bar.baz/my/box/31.29.1/img.box".to_string();
    static ref URL3: String = "https://foo.bar.baz/my/box/28.1/img.box".to_string();
    static ref URL4: String = "https://foo.bar.baz/my/box/29/img.box".to_string();
    static ref BOX_VERSION_1: vagabond::BoxVersion<'static, 'static> = vagabond::BoxVersion {
        version: &VERSION,
        description: &VER_DESCR,
    };
    static ref BOX_VERSION_2: vagabond::BoxVersion<'static, 'static> = vagabond::BoxVersion {
        version: &VERSION2,
        description: &VER_DESCR,
    };
    static ref BOX_VERSION_3: vagabond::BoxVersion<'static, 'static> = vagabond::BoxVersion {
        version: &VERSION3,
        description: &VER_DESCR,
    };
    static ref BOX_VERSION_4: vagabond::BoxVersion<'static, 'static> = vagabond::BoxVersion {
        version: &VERSION4,
        description: &VER_DESCR,
    };
    static ref LIBVIRT_PROVIDER_1: vagabond::BoxProvider<'static, 'static> =
        vagabond::BoxProvider {
            name: &LIBVIRT,
            url: &URL,
        };
    static ref LIBVIRT_PROVIDER_2: vagabond::BoxProvider<'static, 'static> =
        vagabond::BoxProvider {
            name: &LIBVIRT,
            url: &URL2,
        };
    static ref LIBVIRT_PROVIDER_3: vagabond::BoxProvider<'static, 'static> =
        vagabond::BoxProvider {
            name: &LIBVIRT,
            url: &URL3,
        };
    static ref LIBVIRT_PROVIDER_4: vagabond::BoxProvider<'static, 'static> =
        vagabond::BoxProvider {
            name: &LIBVIRT,
            url: &URL4,
        };
    static ref VIRTUALBOX_PROVIDER_1: vagabond::BoxProvider<'static, 'static> =
        vagabond::BoxProvider {
            name: &VIRTUALBOX,
            url: &URL,
        };
}

// fn assert_all_equal(api_response: &vagabond::api::VagrantBox) -> () {}

#[test]
/// this tests ensure_provider_present() by creating a provider from scratch and
/// then compares the resulting API response to the input we gave it
fn test_create_provider_from_empty() {
    let fixture = TestFixture::new(Some(&BOX_NAME));

    let box_res = fixture.client.ensure_provider_present(
        &fixture.get_vagrant_box(),
        &BOX_VERSION_1,
        &LIBVIRT_PROVIDER_1,
        false,
    );

    assert!(box_res.is_ok());

    let box_res = box_res.unwrap();
    assert_eq!(&fixture.get_vagrant_box(), box_res);

    assert_eq!(box_res.versions.len(), 1);
    assert_eq!(&box_res.versions[0], *BOX_VERSION_1);

    assert_eq!(box_res.versions[0].providers.len(), 1);
    assert_eq!(&box_res.versions[0].providers[0], *LIBVIRT_PROVIDER_1);
}

#[test]
/// check that ensure_provider_present() correctly updates the vagrant box if we
/// pass it an updated one
fn check_box_updated_by_ensure_provider_present() {
    let fixture = TestFixture::new(Some(&BOX_NAME));

    let new_box = fixture
        .client
        .ensure_provider_present(
            &fixture.get_vagrant_box(),
            &BOX_VERSION_1,
            &LIBVIRT_PROVIDER_1,
            false,
        )
        .unwrap();

    assert!(new_box.description_markdown.is_none());

    let description = "This is a description".to_string();

    let box_with_description = vagabond::VagrantBox {
        username: &new_box.username,
        name: &new_box.name,
        is_private: new_box.private,
        short_description: new_box.short_description.as_ref(),
        description: Some(&description),
    };

    let updated_box = fixture.client.ensure_provider_present(
        &box_with_description,
        &BOX_VERSION_1,
        &LIBVIRT_PROVIDER_1,
        false,
    );
    assert!(updated_box.is_ok());

    let updated_box = updated_box.unwrap();

    assert!(updated_box.description_markdown.is_some());
    assert_eq!(updated_box.description_markdown, Some(description));
}

#[test]
/// check that ensure_provider_present() correctly updates the provider if we
/// pass it an updated one
fn check_provider_updated_by_ensure_provider_present() {
    let fixture = TestFixture::new(Some(&BOX_NAME));

    let new_box = fixture
        .client
        .ensure_provider_present(
            &fixture.get_vagrant_box(),
            &BOX_VERSION_1,
            &LIBVIRT_PROVIDER_1,
            false,
        )
        .unwrap();

    let old_provider = &new_box.versions[0].providers[0];
    assert_eq!(old_provider, *LIBVIRT_PROVIDER_1);

    let url = "https://this.url.doesn/t/exist.box".to_string();
    let provider_with_new_url = vagabond::BoxProvider {
        name: LIBVIRT_PROVIDER_1.name,
        url: &url,
    };

    let updated_box = fixture.client.ensure_provider_present(
        &fixture.get_vagrant_box(),
        &BOX_VERSION_1,
        &provider_with_new_url,
        false,
    );
    assert!(updated_box.is_ok());

    let updated_box = updated_box.unwrap();

    assert_eq!(updated_box.versions.len(), 1);
    assert_eq!(updated_box.versions[0].providers.len(), 1);

    let updated_provider = &updated_box.versions[0].providers[0];
    assert!(updated_provider.original_url.is_some());
    assert_eq!(updated_provider.original_url.as_ref().unwrap(), &url);
}

#[test]
/// check whether ensure_provider_present() adds a second provider to an already
/// existing version
fn test_add_second_provider() {
    let fixture = TestFixture::new(Some(&BOX_NAME));

    fixture
        .client
        .ensure_provider_present(
            &fixture.get_vagrant_box(),
            &BOX_VERSION_1,
            &LIBVIRT_PROVIDER_1,
            false,
        )
        .unwrap();

    let box_res = fixture
        .client
        .ensure_provider_present(
            &fixture.get_vagrant_box(),
            &BOX_VERSION_1,
            &VIRTUALBOX_PROVIDER_1,
            false,
        )
        .unwrap();

    assert_eq!(&fixture.get_vagrant_box(), box_res);

    assert_eq!(box_res.versions.len(), 1);
    assert_eq!(&box_res.versions[0], *BOX_VERSION_1);

    let prov = &box_res.versions[0].providers;

    assert_eq!(box_res.versions[0].providers.len(), 2);
    assert!(prov.into_iter().any(|prov| prov == *LIBVIRT_PROVIDER_1));
    assert!(prov.into_iter().any(|prov| prov == *VIRTUALBOX_PROVIDER_1));
}

#[test]
/// this tests whether ensure_provider_present() correctly adds a second version
/// with a different provider to a already existing box
fn test_add_second_version() {
    let fixture = TestFixture::new(Some(&BOX_NAME));

    fixture
        .client
        .ensure_provider_present(
            &fixture.get_vagrant_box(),
            &BOX_VERSION_1,
            &LIBVIRT_PROVIDER_1,
            false,
        )
        .unwrap();

    let box_res = fixture
        .client
        .ensure_provider_present(
            &fixture.get_vagrant_box(),
            &BOX_VERSION_2,
            &LIBVIRT_PROVIDER_2,
            false,
        )
        .unwrap();

    assert_eq!(&fixture.get_vagrant_box(), box_res);

    assert_eq!(box_res.versions.len(), 2);
    assert_eq!(&box_res.versions[0], *BOX_VERSION_2);
    assert_eq!(&box_res.versions[1], *BOX_VERSION_1);

    assert_eq!(box_res.versions[0].providers.len(), 1);
    assert_eq!(&box_res.versions[0].providers[0], *LIBVIRT_PROVIDER_2);
}

#[test]
/// Add 3 versions with two providers each in the standard way, except for the
/// version 3 libvirt provider, which is added with delete_other_version=true
/// => there shouldn't be any libvirt providers left anywhere
fn test_remove_all_other_providers() {
    let fixture = TestFixture::new(Some(&BOX_NAME));

    let create_provider = |version, provider| {
        fixture
            .client
            .ensure_provider_present(&fixture.get_vagrant_box(), version, provider, false)
            .unwrap()
    };

    create_provider(&BOX_VERSION_1, &LIBVIRT_PROVIDER_1);
    create_provider(&BOX_VERSION_1, &VIRTUALBOX_PROVIDER_1);

    create_provider(&BOX_VERSION_2, &LIBVIRT_PROVIDER_2);
    create_provider(&BOX_VERSION_2, &VIRTUALBOX_PROVIDER_1);

    create_provider(&BOX_VERSION_3, &VIRTUALBOX_PROVIDER_1);

    create_provider(&BOX_VERSION_4, &LIBVIRT_PROVIDER_4);

    let box_res = fixture
        .client
        .ensure_provider_present(
            &fixture.get_vagrant_box(),
            &BOX_VERSION_3,
            &LIBVIRT_PROVIDER_3,
            true,
        )
        .unwrap();

    assert_eq!(&fixture.get_vagrant_box(), box_res);

    assert_eq!(box_res.versions.len(), 3);

    let ver1_i = box_res
        .versions
        .iter()
        .position(|ver| ver == *BOX_VERSION_1)
        .unwrap();
    let ver2_i = box_res
        .versions
        .iter()
        .position(|ver| ver == *BOX_VERSION_2)
        .unwrap();
    let ver3_i = box_res
        .versions
        .iter()
        .position(|ver| ver == *BOX_VERSION_3)
        .unwrap();

    assert_eq!(box_res.versions[ver1_i].providers.len(), 1);
    assert_eq!(
        &box_res.versions[ver1_i].providers[0],
        *VIRTUALBOX_PROVIDER_1
    );

    assert_eq!(box_res.versions[ver2_i].providers.len(), 1);
    assert_eq!(
        &box_res.versions[ver2_i].providers[0],
        *VIRTUALBOX_PROVIDER_1
    );

    assert_eq!(box_res.versions[ver3_i].providers.len(), 2);
    assert!(&box_res.versions[ver3_i]
        .providers
        .iter()
        .any(|prov| prov == *LIBVIRT_PROVIDER_3));
    assert!(&box_res.versions[ver3_i]
        .providers
        .iter()
        .any(|prov| prov == *VIRTUALBOX_PROVIDER_1));
}

#[test]
/// check that if we create a new provider from scratch and call the function
/// again with delete_other_version=true, that the provider is not deleted
fn ensure_provider_present_doesnt_delete_passed_provider() {
    let fixture = TestFixture::new(Some(&BOX_NAME));

    let create_provider = |version, provider| {
        fixture
            .client
            .ensure_provider_present(&fixture.get_vagrant_box(), version, provider, false)
            .unwrap()
    };

    create_provider(&BOX_VERSION_1, &LIBVIRT_PROVIDER_1);
    create_provider(&BOX_VERSION_1, &VIRTUALBOX_PROVIDER_1);

    let ensure_res = fixture
        .client
        .ensure_provider_present(
            &fixture.get_vagrant_box(),
            &BOX_VERSION_1,
            &LIBVIRT_PROVIDER_1,
            true,
        )
        .unwrap();

    assert_eq!(&fixture.get_vagrant_box(), ensure_res);

    assert_eq!(ensure_res.versions.len(), 1);
    assert_eq!(&ensure_res.versions[0], *BOX_VERSION_1);

    let providers = &ensure_res.versions[0].providers;
    assert_eq!(providers.len(), 2);
    assert!(providers.iter().any(|prov| prov == *LIBVIRT_PROVIDER_1));
    assert!(providers.iter().any(|prov| prov == *VIRTUALBOX_PROVIDER_1));
}

#[test]
fn check_request_without_api_key_works() {
    let client = vagabond::Client::new(None as Option<String>);

    let ubuntu = "ubuntu".to_string();
    let trusty = "trusty64".to_string();
    let trusty64 = vagabond::VagrantBox::new(&ubuntu, &trusty);
    let ubuntu_box = client.read_box(&trusty64);

    assert!(ubuntu_box.is_ok());
    let ubuntu_box = ubuntu_box.unwrap();
    assert_eq!(ubuntu_box.name, trusty);
    assert_eq!(ubuntu_box.username, ubuntu);
}
