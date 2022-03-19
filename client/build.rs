fn main() {
    tonic_build::compile_protos("../server/src/main/protobuf/server.proto")
        .expect("Failed to compile server proto file")
}
