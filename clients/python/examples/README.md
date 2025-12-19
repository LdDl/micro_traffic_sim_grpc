# Python Client Example

This example demonstrates a complete traffic simulation workflow using the Python gRPC client.

## Prerequisites

1. Start the gRPC server (choose one option):

**Docker (recommended):**
```sh
docker run --rm -p 50051:50051 dimahkiin/micro-traffic-sim-server:latest
```

**From source:**
```sh
cargo run --features server --bin micro_traffic_sim
```

2. Set the server address (optional, defaults to `127.0.0.1:50051`):
```sh
export MT_SIM_ADDR=127.0.0.1:50051
```

## Run the Example

From the repository root:

```sh
source clients/python/.venv/bin/activate
python clients/python/examples/main.py > clients/python/examples/output.txt
```

## Generate Visualization

After running the example, generate an animated GIF with gnuplot:

```sh
gnuplot clients/python/examples/plot_anim.gnuplot
```

This creates `clients/python/examples/output.gif`.

## What the Example Does

1. Creates a new simulation session
2. Pushes a grid of 30 cells forming 3 intersecting roads
3. Configures conflict zones at the intersection
4. Sets up a traffic light with 2 signal groups
5. Creates 3 vehicle trip generators (spawning cars, buses, and taxis)
6. Runs 50 simulation steps and outputs vehicle/traffic light states

The output format is compatible with the gnuplot script for visualization.
