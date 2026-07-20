mod utils;

use crate::utils::server_analyze;
use mc_protocol::types::{McVersion, McVersionEnum, Player};
use testcontainers::ImageExt;
use testcontainers::runners::AsyncRunner;
use utils::containers::generate_container;
use utils::server_analyze::ConnectionExitReason;

async fn test(version: McVersion) {
    let container = generate_container(version, "vanilla").with_env_var("ONLINE_MODE", "TRUE");
    let container = container.start().await.unwrap();
    let host_port = container.get_host_port_ipv4(25565).await.unwrap();
    let player = Player::random_like_offline();
    let result =
        server_analyze::parse_server_data(format!("127.0.0.1:{}", host_port), &player).await;
    dbg!(&result);
    match result.exit_reason {
        ConnectionExitReason::Encryption => {}
        exit_reason => panic!("Expected encryption exit, but got {:?}", exit_reason),
    }
    let server_data = result.server_data;
    assert_eq!(server_data.protocol, version.protocol);
}
#[tokio::test]
async fn test_vanilla_26_2_online() {
    let version = McVersionEnum::V26_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_26_1_2_online() {
    let version = McVersionEnum::V26_1_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_21_11_online() {
    let version = McVersionEnum::V1_21_11.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_21_10_online() {
    let version = McVersionEnum::V1_21_10.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_21_8_online() {
    let version = McVersionEnum::V1_21_8.data();
    test(version).await;
}

#[tokio::test]
async fn test_vanilla_1_21_6_online() {
    let version = McVersionEnum::V1_21_6.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_21_5_online() {
    let version = McVersionEnum::V1_21_5.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_21_4_online() {
    let version = McVersionEnum::V1_21_4.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_21_3_online() {
    let version = McVersionEnum::V1_21_3.data();
    test(version).await;
}

#[tokio::test]
async fn test_vanilla_1_21_1_online() {
    let version = McVersionEnum::V1_21_1.data();
    test(version).await;
}

#[tokio::test]
async fn test_vanilla_1_20_6_online() {
    let version = McVersionEnum::V1_20_6.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_20_4_online() {
    let version = McVersionEnum::V1_20_4.data();
    test(version).await;
}

#[tokio::test]
async fn test_vanilla_1_20_2_online() {
    let version = McVersionEnum::V1_20_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_20_1_online() {
    let version = McVersionEnum::V1_20_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_19_4_online() {
    let version = McVersionEnum::V1_19_4.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_19_3_online() {
    let version = McVersionEnum::V1_19_3.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_19_2_online() {
    let version = McVersionEnum::V1_19_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_19_online() {
    let version = McVersionEnum::V1_19.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_18_2_online() {
    let version = McVersionEnum::V1_18_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_18_1_online() {
    let version = McVersionEnum::V1_18_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_17_1_online() {
    let version = McVersionEnum::V1_17_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_17_online() {
    let version = McVersionEnum::V1_17.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_16_5_online() {
    let version = McVersionEnum::V1_16_5.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_16_3_online() {
    let version = McVersionEnum::V1_16_3.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_16_2_online() {
    let version = McVersionEnum::V1_16_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_16_1_online() {
    let version = McVersionEnum::V1_16_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_16_online() {
    let version = McVersionEnum::V1_16.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_15_2_online() {
    let version = McVersionEnum::V1_15_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_15_1_online() {
    let version = McVersionEnum::V1_15_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_15_online() {
    let version = McVersionEnum::V1_15.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_14_4_online() {
    let version = McVersionEnum::V1_14_4.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_14_3_online() {
    let version = McVersionEnum::V1_14_3.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_14_2_online() {
    let version = McVersionEnum::V1_14_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_14_1_online() {
    let version = McVersionEnum::V1_14_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_14_online() {
    let version = McVersionEnum::V1_14.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_13_2_online() {
    let version = McVersionEnum::V1_13_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_13_1_online() {
    let version = McVersionEnum::V1_13_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_13_online() {
    let version = McVersionEnum::V1_13.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_12_2_online() {
    let version = McVersionEnum::V1_12_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_12_1_online() {
    let version = McVersionEnum::V1_12_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_12_online() {
    let version = McVersionEnum::V1_12.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_11_2_online() {
    let version = McVersionEnum::V1_11_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_11_online() {
    let version = McVersionEnum::V1_11.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_10_2_online() {
    let version = McVersionEnum::V1_10_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_10_1_online() {
    let version = McVersionEnum::V1_10_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_10_online() {
    let version = McVersionEnum::V1_10.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_9_4_online() {
    let version = McVersionEnum::V1_9_4.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_9_2_online() {
    let version = McVersionEnum::V1_9_2.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_9_1_online() {
    let version = McVersionEnum::V1_9_1.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_9_online() {
    let version = McVersionEnum::V1_9.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_8_9_online() {
    let version = McVersionEnum::V1_8_9.data();
    test(version).await;
}
#[tokio::test]
async fn test_vanilla_1_7_10_online() {
    let version = McVersionEnum::V1_7_10.data();
    test(version).await;
}
