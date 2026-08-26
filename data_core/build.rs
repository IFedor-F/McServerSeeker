fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .type_attribute(
            ".scanner.ScanMethod",
            "#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]",
        )
        .type_attribute(
            ".scanner.ScanMethod",
            "#[serde(rename_all = \"snake_case\")]",
        )
        .compile_protos(&["proto/scanner.proto"], &["proto"])?;
    Ok(())
}
