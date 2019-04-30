extern crate stderrlog;

extern crate vagabond;

#[macro_use]
extern crate log;

use std::env;
use stderrlog::*;

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
        let fixture = TestFixture {
            client: vagabond::Client::new(Some(env::var("ATLAS_TOKEN").unwrap())),
            user: env::var("ATLAS_USER").unwrap(),
            box_name: box_name.map_or("test_box".to_string(), |b| b.to_string()),
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

    // fn version_create<S, T>(
    //     &self,
    //     version_str: S,
    //     description: T,
    // ) -> vagabond::Result<vagabond::api::Version>
    // where
    //     S: Into<String>,
    //     T: Into<String>,
    // {
    //     let ver_str = version_str.into();
    //     let descr = description.into();
    //     let version = vagabond::BoxVersion {
    //         version: &ver_str,
    //         description: &descr,
    //     };
    //     self.client
    //         .create_version(&self.get_vagrant_box(), &version)
    // }
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
