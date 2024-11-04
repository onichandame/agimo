use std::path::PathBuf;

fn main() {
    let proto_file = "./proto/externalscaler.proto";
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .build_server(true)
        .file_descriptor_set_path(out_dir.join("externalscaler.bin"))
        .out_dir("./src")
        .compile(&[proto_file], &["proto"])
        .unwrap();
}
