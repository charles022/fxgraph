fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use vendored protoc so the build works without a system install.
    let protoc_path = protoc_bin_vendored::protoc_bin_path()?;
    std::env::set_var("PROTOC", protoc_path);

    tonic_build::configure().compile_protos(&["../proto/dashboard.proto"], &["../proto"])?;
    Ok(())
}
