# micro_traffic_sim gRPC server

[![Crates.io](https://img.shields.io/crates/v/micro_traffic_sim.svg)](https://crates.io/crates/micro_traffic_sim)
[![Documentation](https://docs.rs/micro_traffic_sim/badge.svg)](https://docs.rs/micro_traffic_sim)
[![License](https://img.shields.io/crates/l/micro_traffic_sim.svg)](https://github.com/LdDl/micro_traffic_sim_grpc/blob/master/LICENSE)

This crate exposes the gRPC API for the micro traffic simulation via cellular automata. It can be used as a Rust library ([crates.io](https://crates.io/crates/micro_traffic_sim)), run as a server binary, and distributed via Docker. Go and Python client stubs can be generated from the same protos.

## Table of Contents
- [Prerequisites for building from source](#prerequisites-for-building-from-source)
- [Build and run (binary)](#build-and-run-binary)
- [Docker](#docker)
    - [Build and run locally](#build-and-run-locally)
    - [Pre-built image from registry](#pre-built-image-from-registry)
- [Pre-built binaries from GitHub releases page](#pre-built-binaries-from-github-releases-page)
- [Usage](#usage)
    - [Run server locally](#run-server-locally)
    - [Rust client to micro_traffic_sim gRPC server](#rust-client-to-micro_traffic_sim-grpc-server)
    - [Golang client to micro_traffic_sim gRPC server](#golang-client-to-micro_traffic_sim-grpc-server)
    - [Python client to micro_traffic_sim gRPC server](#python-client-to-micro_traffic_sim-grpc-server)
- [Client code generation](#client-code-generation)
    - [Golang](#golang)
    - [Python](#python)

## Prerequisites for building from source

- Rust 1.91.0 which is tested with 2024 edition in my case
- `protoc` available on PATH
- Optional: `docker` for container builds

## Build and run (binary)

- Debug build (library mode):
  - `make build`
- Run the gRPC server:
  - `make run-server`
- Release binary:
  - `make build-release`
  - Binary path: `target/release/micro_traffic_sim`

Notes:
- The server is behind a Cargo feature flag `server`. Commands above enable it when needed. Those are basically:
```sh
cargo build --release --features server
```
- Default listen address is `0.0.0.0:50051`.

## Docker

There are two supported paths: build locally with Dockerfile, or pull from registry.

### Build and run locally

- Build
  - `make docker-build IMAGE=micro-traffic-sim/server TAG=latest`
- Run
  - `make docker-run IMAGE=micro-traffic-sim/server TAG=latest`
  - This maps host port 50051 -> container port 50051.

The Docker image is built with a multi-stage process (Rust builder + slim runtime). It compiles with the `server` feature enabled.

### Pre-built image from registry

@todo

## Pre-built binaries from GitHub releases page

@todo

## Usage

### Run server locally

E.g. we can run the server in debug mode with:

```sh
cargo run --features server --bin micro_traffic_sim
```

### Rust client to micro_traffic_sim gRPC server

Add the crate to your project: `cargo add micro_traffic_sim`

- [API Documentation (docs.rs)](https://docs.rs/micro_traffic_sim)
- [Example details](./examples/rust_client/README.md)

```sh
export MT_SIM_ADDR=127.0.0.1:50051
cargo run --example rust_client   
```

### Golang client to micro_traffic_sim gRPC server

Here more details: [clients/go/cmd/example/README.md](./clients/go/cmd/example/README.md)

```sh
export MT_SIM_ADDR=127.0.0.1:50051
# from repository root
cd ./clients/go
go run ./cmd/example/main.go
```

### Python client to micro_traffic_sim gRPC server

Here more details: [clients/python/README.md](./clients/python/README.md)

```sh
export MT_SIM_ADDR=127.0.0.1:50051
# from repository root
cd ./clients/python
source .venv/bin/activate
python examples/main.py
```

## Client code generation

This section describes how I've used to generate client code for different languages from the proto files.

### Golang
Client code generation for Golang is done via [scripts/gen_go.sh](./scripts/gen_go.sh). It requires `protoc` and `protoc-gen-go` to be installed and available on PATH.
```sh
chmod +x ./scripts/gen_go.sh
./scripts/gen_go.sh clients/go
cd ./clients/go
go mod init github.com/LdDl/micro_traffic_sim_grpc/clients/go
go mod tidy
cd -
```

### Python

Client code generation for Python is done via [scripts/gen_python.sh](./scripts/gen_python.sh). The script automatically creates a virtual environment and installs all dependencies.

```sh
chmod +x ./scripts/gen_python.sh
./scripts/gen_python.sh
```

The script:
1. Creates `.venv` in `clients/python/` (if not exists)
2. Installs dependencies from `requirements.txt`
3. Generates `*_pb2.py`, `*_pb2.pyi` (type stubs), and `*_pb2_grpc.py`
4. Installs the `micro-traffic-sim` package in editable mode