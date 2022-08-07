use protobuf_codegen::Codegen;
use std::{env, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let proto_dir = manifest_dir.join("protos");
    let out_dir = manifest_dir.join("src").join("generated");

    if !out_dir.exists() {
        std::fs::create_dir_all(&out_dir).unwrap();
    };

    let files = vec![
        proto_dir.join("connectivity.proto"),
        proto_dir.join("spotify/client_token/v0/client_token_http.proto"),
    ];

    let proto_dir = proto_dir.to_str().expect("Ok");

    Codegen::new()
        .out_dir(out_dir)
        .inputs(files.as_slice())
        .include(proto_dir)
        .run()
        .expect("protoc");

    Ok(())
}
