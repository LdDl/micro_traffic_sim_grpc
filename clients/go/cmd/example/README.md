# Go client to micro_traffic_sim gRPC server

## Run server locally

To run the server locally, follow the instructions in the [micro_traffic_sim gRPC server README](../../README.md) to build and run the server binary.

E.g. run the server in debug mode with:

```sh
cargo run --features server --bin micro_traffic_sim
```

After that you should see:
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
    Running `target/debug/micro_traffic_sim`
Starting micro_traffic_sim gRPC server on 0.0.0.0:50051
```

## Run client example in Go

```sh
export MT_SIM_ADDR=127.0.0.1:50051
cd clients/go
go run ./cmd/example
```

Expected output:
```
New session created:
  code: 0 text: The operation completed successfully
  id:   d4f68898-c1b0-4586-88ee-b// filepath: /home/dimitrii/rust_work/micro_traffic_sim_grpc/clients/go/README.md
# Go client to micro_traffic_sim gRPC server

## Run server locally

To run the server locally, follow the instructions in the [micro_traffic_sim gRPC server README](../../README.md) to build and run the server binary.

E.g. run the server in debug mode with:

```sh
cargo run --features server --bin micro_traffic_sim
```

After that you should see:
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
    Running `target/debug/micro_traffic_sim`
Starting micro_traffic_sim gRPC server on 0.0.0.0:50051
```

## Run client example in Go

```sh
export MT_SIM_ADDR=127.0.0.1:50051
cd clients/go
go run ./cmd/example
```

If everything is working correctly, you should see output similar to:
```
New session created:
  code: 0 text: The operation completed successfully
  id:   49f2b649-3f63-4397-a7a3-b7990d5a1d2e
Info session:
  code: 0 text: The operation completed successfully
  id:   49f2b649-3f63-4397-a7a3-b7990d5a1d2e
```

## go.mod

Add the following to your `go.mod` to use the your Golang-based client outside of the examples:

```
require github.com/LdDl/micro_traffic_sim_grpc/clients/go v0.1.0
```