extern crate speculate;
extern crate vagabond;

use speculate::speculate;

use std::env;

speculate! {

    before {
        let client: vagabond::Client =
            vagabond::Client::new(Some(env::var("ATLAS_TOKEN").unwrap()));
        let user: String = env::var("ATLAS_USER").unwrap();
        let box_name: String = "test_box_1".to_string();

        // try to delete the test box in case it still exists
        let vagrant_box = vagabond::VagrantBox::new(&user, &box_name);
        client.delete_box(&vagrant_box);

        let box_create = || { client.create_box(&vagrant_box) };
        let version_create = |version| { client.create_version(&vagrant_box, version) };
    }

    test "Box creation should succeed" {
        let box_create_res = box_create();

        assert!(box_create_res.is_ok());

        let box_res = box_create_res.unwrap();
        assert_eq!(box_res.username, user);
        assert_eq!(box_res.name, box_name);
    }

    test "Version creation should succeed" {
        box_create().unwrap();

        let version_str: String = "1.2.3".to_string();
        let description: String = "This is a test box".to_string();
        let version = vagabond::BoxVersion {
            version: &version_str, description: &description
        };

        let ver_create_res = version_create(&version);
        assert!(ver_create_res.is_ok());
    }

    // describe "Create a version" {

    //     before {
    //         let vagrant_box = vagabond::VagrantBox::new(&user, &box_name);
    //         let box_create_res = client
    //             .create_box(&vagrant_box)
    //             .expect("Box creation should have succeeded.");

    //     }

    //     test "Version creation should succeed" {
    //         let version_create_res = client.create_version(&vagrant_box, &version);
    //         assert!(version_create_res.is_ok());

    //         let version_res = version_create_res.unwrap();
    //         assert_eq!(version_res.version, version_str);
    //     }

    //     describe "Manipulate a version" {

    //         before {
    //             client
    //                 .create_version(&vagrant_box, &version)
    //                 .expect("Version creation should succeed");
    //         }

    //         test "Version deletion should succeed" {
    //             let version_delete_res = client.delete_version(&vagrant_box, &version);
    //             assert!(version_delete_res.is_ok());
    //         }

    //         // after {
    //         //     client
    //         //         .delete_version(&vagrant_box, &version)
    //         //         .expect("Version deletion should succeed");
    //         // }
    //     }
    // }

    after {
        client.delete_box(&vagrant_box);
    }
}
