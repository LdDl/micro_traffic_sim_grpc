// Public library surface for consumers (Rust server/client and external crates)
// Generated code is included from OUT_DIR; tonic/prost generate a single file per proto package.
// Current proto package name is `micro_traffic_sim`, so the generated file is typically `micro_traffic_sim.rs`.
// If you version the package later (e.g., micro_traffic_sim.v1), adjust the include! path accordingly.

// Re-export generated protobuf code under a stable module name.
pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/micro_traffic_sim.rs"));
}

// Convenient re-exports for common types at crate root. Depending on how prost/tonic
// structured the generated module, items may live at the top of `pb` or under a
// nested `micro_traffic_sim` module. Re-export everything from `pb` to keep this
// resilient to generation differences.
pub use pb::*;
