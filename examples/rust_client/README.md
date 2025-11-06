## Rust client to micro_traffic_sim gRPC server

## Run server locally

To run the server locally, follow the instructions in the [micro_traffic_sim gRPC server README](../../README.md) to build and run the server binary.

E.g. we can run the server in debug mode with:

```sh
cargo run --features server --bin micro_traffic_sim
```

After that we can see following in the shell:
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
    Running `target/debug/micro_traffic_sim`
Starting micro_traffic_sim gRPC server on 0.0.0.0:50051
```

## Run client example in Rust
```
export MT_SIM_ADDR=127.0.0.1:50051
cargo run --example rust_client
```

If everything is working correctly, you should see output similar to:
```
New session created:
  code: 0 text: The operation completed successfully
  id:   d4f68898-c1b0-4586-88ee-bbe2349cd491
Info session:
  code: 0 text: The operation completed successfully
  id:   d4f68898-c1b0-4586-88ee-bbe2349cd491
```

## Cargo.toml

Add the following to your `Cargo.toml` to use the your Rust-based client outside of the examples:

```toml
[package]
name = "micro_traffic_sim_rust_client"
version = "0.1.0"
edition = "2024"

[dependencies]
micro_traffic_sim = { version = "0.1.0" }
tonic = { version = "0.14.2", features = ["transport"] }
tokio = { version = "1.40", features = ["macros", "rt-multi-thread"] }
```