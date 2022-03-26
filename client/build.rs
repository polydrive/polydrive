fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir =
        std::env::var("PROTO_PATH").unwrap_or_else(|_| "../server/src/main/protobuf".to_string());
    let protos =
        ["server", "file", "client", "upload"].map(|file| format!("{}/{}.proto", &proto_dir, file));

    tonic_build::configure()
        .build_server(false)
        .compile(&protos, &[&proto_dir])?;
    Ok(())
}
