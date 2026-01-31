use tonic_prost_build::configure;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    configure().compile_protos(
        &["../proto/control.proto", "../proto/monitord.proto"],
        &["../proto"],
    )?;
    Ok(())
}
