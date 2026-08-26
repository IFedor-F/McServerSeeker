mod containers;

use containers::generate_container;
use mc_protocol::dialog::{
    ConnectionMethod, ConnectionResult, ConnectionSettings, ResourcePack, ServerDialog, ServerDst,
};
use mc_protocol::types::{McVersion, McVersionEnum, Player};
use testcontainers::ImageExt;
use testcontainers::runners::AsyncRunner;

async fn test(version: McVersion) {
    let _ = env_logger::builder().is_test(true).try_init();
    let mut container = generate_container(version, "vanilla");
    container = container
        .with_env_var("ONLINE_MODE", "FALSE")
        .with_env_var("RESOURCE_PACK", "https://some_link.example/to/pack.zip?=1")
        .with_env_var(
            "RESOURCE_PACK_SHA1",
            "d5db29cd03a2ed055086cef9c31c252b4587d6d0",
        )
        .with_env_var("RESOURCE_PACK_ENFORCE", "TRUE")
        .with_env_var(
            "CUSTOM_SERVER_PROPERTIES",
            "resource-pack-prompt=\"prompt\"\nbug-report-link=https://bug-report-link.example",
        );

    let container = container.start().await.unwrap();
    let host_port = container.get_host_port_ipv4(25565).await.unwrap();
    let player = Player::random_like_offline();
    let dst = ServerDst::from_ip_and_port("127.0.0.1".parse().unwrap(), host_port);
    let dialog = ServerDialog::new(dst, player.clone());
    let conn_settings = ConnectionSettings {
        conn_method: ConnectionMethod::Join,
        ..Default::default()
    };
    let result = dialog.connect(conn_settings).await;
    dbg!(&result);
    let data = match result {
        ConnectionResult::Successful { data } => data,
        another => panic!("expect successful, but got {:?}", another),
    };

    assert_eq!(data.protocol, version.protocol);
    if version >= McVersionEnum::V1_9.data() {
        assert_eq!(
            data.resource_pack,
            Some(ResourcePack {
                url: "https://some_link.example/to/pack.zip?=1".to_string(),
                hash: Some("d5db29cd03a2ed055086cef9c31c252b4587d6d0".to_string()),
                forced: true,
            })
        );
        // in 1.8.9 for some reason hash string is empty, so we skip test for this version
    } else if version <= McVersionEnum::V1_7_10.data() {
        assert_eq!(
            data.resource_pack,
            Some(ResourcePack {
                url: "https://some_link.example/to/pack.zip?=1".to_string(),
                hash: None,
                forced: true,
            })
        );
    }
    if version >= McVersionEnum::V1_21.data() {
        assert_eq!(data.links, vec!["https://bug-report-link.example"]);
    }
}
#[tokio::test]
async fn test_vanilla_26_2_offline() {
    let version = McVersionEnum::V26_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_26_1_2_offline() {
    let version = McVersionEnum::V26_1_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_21_11_offline() {
    let version = McVersionEnum::V1_21_11.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_21_10_offline() {
    let version = McVersionEnum::V1_21_10.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_21_8_offline() {
    let version = McVersionEnum::V1_21_8.data();
    test(version).await;
}

#[tokio::test]
async fn test_vanilla_1_21_6_offline() {
    let version = McVersionEnum::V1_21_6.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_21_5_offline() {
    let version = McVersionEnum::V1_21_5.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_21_4_offline() {
    let version = McVersionEnum::V1_21_4.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_21_3_offline() {
    let version = McVersionEnum::V1_21_3.data();
    test(version).await;
}

#[tokio::test]
async fn test_vanilla_1_21_1_offline() {
    let version = McVersionEnum::V1_21_1.data();
    test(version).await;
}

#[tokio::test]
async fn test_vanilla_1_20_6_offline() {
    let version = McVersionEnum::V1_20_6.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_20_4_offline() {
    let version = McVersionEnum::V1_20_4.data();
    test(version).await;
}

#[tokio::test]
async fn test_vanilla_1_20_2_offline() {
    let version = McVersionEnum::V1_20_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_20_1_offline() {
    let version = McVersionEnum::V1_20_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_19_4_offline() {
    let version = McVersionEnum::V1_19_4.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_19_3_offline() {
    let version = McVersionEnum::V1_19_3.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_19_2_offline() {
    let version = McVersionEnum::V1_19_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_19_offline() {
    let version = McVersionEnum::V1_19.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_18_2_offline() {
    let version = McVersionEnum::V1_18_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_18_1_offline() {
    let version = McVersionEnum::V1_18_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_17_1_offline() {
    let version = McVersionEnum::V1_17_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_17_offline() {
    let version = McVersionEnum::V1_17.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_16_5_offline() {
    let version = McVersionEnum::V1_16_5.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_16_3_offline() {
    let version = McVersionEnum::V1_16_3.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_16_2_offline() {
    let version = McVersionEnum::V1_16_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_16_1_offline() {
    let version = McVersionEnum::V1_16_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_16_offline() {
    let version = McVersionEnum::V1_16.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_15_2_offline() {
    let version = McVersionEnum::V1_15_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_15_1_offline() {
    let version = McVersionEnum::V1_15_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_15_offline() {
    let version = McVersionEnum::V1_15.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_14_4_offline() {
    let version = McVersionEnum::V1_14_4.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_14_3_offline() {
    let version = McVersionEnum::V1_14_3.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_14_2_offline() {
    let version = McVersionEnum::V1_14_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_14_1_offline() {
    let version = McVersionEnum::V1_14_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_14_offline() {
    let version = McVersionEnum::V1_14.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_13_2_offline() {
    let version = McVersionEnum::V1_13_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_13_1_offline() {
    let version = McVersionEnum::V1_13_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_13_offline() {
    let version = McVersionEnum::V1_13.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_12_2_offline() {
    let version = McVersionEnum::V1_12_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_12_1_offline() {
    let version = McVersionEnum::V1_12_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_12_offline() {
    let version = McVersionEnum::V1_12.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_11_2_offline() {
    let version = McVersionEnum::V1_11_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_11_offline() {
    let version = McVersionEnum::V1_11.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_10_2_offline() {
    let version = McVersionEnum::V1_10_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_10_1_offline() {
    let version = McVersionEnum::V1_10_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_10_offline() {
    let version = McVersionEnum::V1_10.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_9_4_offline() {
    let version = McVersionEnum::V1_9_4.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_9_2_offline() {
    let version = McVersionEnum::V1_9_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_9_1_offline() {
    let version = McVersionEnum::V1_9_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_9_offline() {
    let version = McVersionEnum::V1_9.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_8_9_offline() {
    let version = McVersionEnum::V1_8_9.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_7_10_offline() {
    let version = McVersionEnum::V1_7_10.data();
    test(version).await;
}
