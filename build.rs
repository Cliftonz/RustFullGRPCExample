use std::env;
use std::path::PathBuf;
use tonic_build;

fn main () -> Result<(), Box<dyn std::error::Error>> {
   // env::set_var("PROTOC", protobuf_src::protoc());
    let proto_file = "protos/voting.proto";
    let descriptor_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("voting_descriptor.bin");
    tonic_build::configure()
        .build_server(true)
        .file_descriptor_set_path(&descriptor_path)
        //.out_dir("./protos/complied")
        .compile(&[proto_file], &[env::current_dir()?]) //&["."])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));
    println!("cargo:rerun-if-changed={}", proto_file);
    Ok(())
}