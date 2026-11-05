//! Compiles the vendored protos with protox, so no target needs `protoc`.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto");
    let fds = protox::compile(["hedra/sandbox/v1/session.proto"], ["proto"])?;
    tonic_prost_build::configure()
        .build_server(cfg!(feature = "fake-broker"))
        .build_client(true)
        .compile_fds(fds)?;
    Ok(())
}
