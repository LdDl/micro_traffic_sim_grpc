.PHONY: gen-go gen-python clean-gen build run run-server build-release docker-build docker-run docker-push

ROOT_DIR := $(shell pwd)
PROTO_DIR := $(ROOT_DIR)/protos

build:
	cargo build

run:
	cargo run

# Run the gRPC server (feature-gated)
run-server:
	cargo run --features server

# Build a release binary with the server enabled
build-release:
	cargo build --release --features server

gen-go:
	bash scripts/gen_go.sh

gen-python:
	bash scripts/gen_python.sh

clean-gen:
	rm -rf gen/go gen/python

# Docker
IMAGE ?= micro-traffic-sim/server
TAG ?= latest
REG ?=

docker-build:
	docker build -f Dockerfile.server -t $(if $(REG),$(REG)/,)$(IMAGE):$(TAG) .

docker-run:
	docker run --rm -p 50051:50051 $(if $(REG),$(REG)/,)$(IMAGE):$(TAG)

docker-push:
	@if [ -z "$(REG)" ]; then echo "Set REG=your-registry.example.com"; exit 1; fi
	docker push $(REG)/$(IMAGE):$(TAG)
