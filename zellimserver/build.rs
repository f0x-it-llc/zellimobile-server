// build.rs — compile proto/zellimserver.proto using tonic-prost-build.
//
// The generated code lands in $OUT_DIR/zellimserver.v1.rs and is included
// via the `include_proto!` macro in src/lib.rs.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("proto/zellimserver.proto")?;
    // Re-run this build script whenever the .proto changes.
    println!("cargo:rerun-if-changed=proto/zellimserver.proto");
    Ok(())
}
