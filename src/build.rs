use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Compile all protos in one shot; tonic/prost will emit a single
    // Rust module per proto package into OUT_DIR (default behavior).
    let protos: [&str; 8] = [
        "protos/service.proto",
        "protos/cell.proto",
        "protos/session.proto",
        "protos/step.proto",
        "protos/trip.proto",
        "protos/tls.proto",
        "protos/conflict_zones.proto",
        "protos/uuid.proto",
    ];

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("sim_service.bin"))
        .compile_protos(&protos, &["protos"]) // OUT_DIR is used implicitly
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));

    // Ensure rebuild when any proto changes
    for p in &protos {
        println!("cargo:rerun-if-changed={}", p);
    }
}
