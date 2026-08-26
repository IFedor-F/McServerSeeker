use dotenvy::dotenv;
use mc_protocol::types::{McVersion, McVersionEnum};
use std::env;
use std::time::Duration;
use testcontainers::core::Mount;
use testcontainers::{
    ContainerRequest, GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
};

pub fn generate_container(version: McVersion, core_type: &str) -> ContainerRequest<GenericImage> {
    dotenv().ok();
    let tag = {
        if version < McVersionEnum::V1_17.data() {
            "java8"
        } else if version < McVersionEnum::V1_18.data() {
            "java17"
        } else if version < McVersionEnum::V26_1.data() {
            "java21"
        } else {
            "latest"
        }
    };
    println!("Use itzg/minecraft-server:{}", tag);
    let mut container = GenericImage::new("itzg/minecraft-server", tag)
        .with_wait_for(WaitFor::healthcheck())
        .with_exposed_port(25565.tcp())
        .with_env_var("EULA", "TRUE")
        .with_env_var("VERSION", version.name())
        .with_env_var("DIFFICULTY", "easy")
        .with_env_var("LEVEL_TYPE", "FLAT");

    // Local files
    let local_server_jars_folder = env::var("LOCAL_SERVER_JARS_FOLDER");
    match local_server_jars_folder {
        Ok(path) => {
            let version_path = format!(
                "{}/{}/{}.jar",
                path,
                core_type.to_lowercase(),
                version.name()
            );
            // Check if the local jar file actually exists
            if std::path::Path::new(&version_path).exists() {
                println!("Use local server jar: {}", version_path);
                container = container
                    .with_mount(Mount::bind_mount(&version_path, "/data/server.jar"))
                    .with_env_var("TYPE", "CUSTOM")
                    .with_env_var("CUSTOM_SERVER", "/data/server.jar");
            } else {
                println!("File not found: {}. Switching to download.", version_path);
                container = container.with_env_var("TYPE", core_type.to_uppercase());
            }
        }
        Err(_) => {
            println!("Server type will be downloaded in docker container");
            container = container.with_env_var("TYPE", core_type.to_uppercase());
        }
    }
    container
        .with_container_name(format!("McServerSeeker_{}_{}", core_type, version.name()))
        .with_startup_timeout(Duration::from_mins(5))
}
