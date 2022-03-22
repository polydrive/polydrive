fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().build_server(false).compile(
        &[
            "../server/src/main/protobuf/server.proto",
            "../server/src/main/protobuf/file.proto",
            "../server/src/main/protobuf/client.proto",
        ],
        &["../server/src/main/protobuf/"],
    )?;
    Ok(())
}
