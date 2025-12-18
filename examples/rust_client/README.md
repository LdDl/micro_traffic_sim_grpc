# Rust client example for micro_traffic_sim gRPC Server

This example demonstrates a complete traffic simulation workflow using the Rust gRPC client.

## Prerequisites

1. Start the gRPC server:
```sh
cargo run --features server --bin micro_traffic_sim
```

2. Set the server address (optional, defaults to `127.0.0.1:50051`):
```sh
export MT_SIM_ADDR=127.0.0.1:50051
```

## Run the example

From the repository root:

```sh
cargo run --example rust_client > examples/rust_client/output.txt
```

## Generate visualization

After running the example, generate an animated GIF with gnuplot:

```sh
gnuplot examples/rust_client/plot_anim.gnuplot
```

This creates `examples/rust_client/output.gif`.

## What the example does

1. Creates a new simulation session
2. Pushes a grid of 30 cells forming 3 intersecting roads
3. Configures conflict zones at the intersection
4. Sets up a traffic light with 2 signal groups
5. Creates 3 vehicle trip generators (spawning cars, buses, and taxis)
6. Runs 50 simulation steps and outputs vehicle/traffic light states

The output format is compatible with the gnuplot script for visualization.

## Using as a library

Add the following to your `Cargo.toml`:

```toml
[dependencies]
micro_traffic_sim = "0.0.1"
tonic = { version = "0.14.2", features = ["transport"] }
tokio = { version = "1.40", features = ["macros", "rt-multi-thread"] }
```

See the [crate documentation](https://docs.rs/micro_traffic_sim) for API details.
