fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compiles the proto file into Rust code
    tonic_build::compile_protos("../proto/dashboard.proto")?;
    Ok(())
}
