# micro_traffic_sim gRPC server

This crate exposes the gRPC API for the micro traffic simulation via cellular automata. It can be used as a Rust library, run as a server binary, and distributed via Docker. Go and Python client stubs can be generated from the same protos.

## Table of Contents
- [Prerequisites for building from source](#prerequisites-for-building-from-source)
- [Build and run (binary)](#build-and-run-binary)
- [Docker](#docker)
    - [Build and run locally](#build-and-run-locally)
    - [Pre-built image from registry](#pre-built-image-from-registry)
- [Pre-built binaries from GitHub releases page](#pre-built-binaries-from-github-releases-page)
- [Client code generation](#client-code-generation)

## Prerequisites for building from source

- Rust 1.9.0 which is tested with 2024 edition
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

## Client code generation

@todo
