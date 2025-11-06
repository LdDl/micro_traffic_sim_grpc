#[cfg(feature = "server")]
mod server;

#[cfg(not(feature = "server"))]
fn main() {
    println!("micro_traffic_sim crate built as a library. Enable the 'server' feature to run the gRPC server.");
}

#[cfg(feature = "server")]
fn main() {
    server::run_blocking();
}
