lazy_static! {
    static ref PROVIDER_LIBVIRT: String = "libvirt".to_string();
    static ref PROVIDER_VIRTBOX: String = "virtualbox".to_string();
    static ref URL: String = "https://foo.bar.baz/my/box/img.box".to_string();
    static ref VERSION: String = "5.6.8".to_string();
    static ref VERSION_DESCRIPTION: String = "The best version to come!".to_string();
    static ref USERNAME: String = "me".to_string();
    static ref BOXNAME: String = "MY_BOX".to_string();
}

use super::*;

#[test]
fn compare_providers() {
    let box_provider = BoxProvider {
        name: &PROVIDER_LIBVIRT,
        url: &URL,
    };

    let mut api_response = api::Provider {
        name: "libvirt".to_string(),
        original_url: Some(URL.to_string()),
        ..Default::default()
    };

    assert_eq!(&box_provider, api_response);

    api_response.name = "bla".to_string();
    assert_ne!(&box_provider, api_response);

    api_response.name = PROVIDER_LIBVIRT.to_string();
    api_response.original_url = None;
    assert_ne!(&box_provider, api_response);
}

#[test]
fn compare_versions() {
    let box_version = BoxVersion {
        version: &VERSION,
        description: &VERSION_DESCRIPTION,
    };

    let mut api_response = api::Version {
        version: VERSION.to_string(),
        ..Default::default()
    };

    assert_ne!(&box_version, api_response);

    api_response.description_markdown = Some(VERSION_DESCRIPTION.to_string());
    assert_eq!(&box_version, api_response);

    api_response.version = "1.2.3".to_string();
    assert_ne!(&box_version, api_response);
}

#[test]
fn compare_boxes() {
    let vagrant_box = VagrantBox::new(&USERNAME, &BOXNAME);

    let mut api_response = api::VagrantBox {
        username: USERNAME.to_string(),
        name: BOXNAME.to_string(),
        ..Default::default()
    };

    assert_eq!(&vagrant_box, api_response);
    assert_eq!(&api_response, vagrant_box);

    api_response.private = Some(true);
    assert_ne!(&vagrant_box, api_response);
}
