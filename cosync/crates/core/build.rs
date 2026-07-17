fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let workspace_dir = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let proto_path = workspace_dir.join("proto/cosync.proto");
    let proto_dir = workspace_dir.join("proto/");

    println!("cargo:rerun-if-changed={}", proto_path.display());

    prost_build::Config::new()
        .compile_protos(&[proto_path], &[proto_dir])
        .expect("Failed to compile protobuf definitions");
}