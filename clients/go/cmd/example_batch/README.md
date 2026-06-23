# Go batch recording example

This example mirrors [`example`](../example) but, instead of stepping the
simulation tick by tick and pulling state back on every tick, it runs the whole
horizon server-side in a single `RunAndRecord` streaming call and decodes the
columnar recording it returns.

The session setup (grid, conflict zones, traffic light, trips) and the output
format are identical to the step-by-step example - only the run call differs, so
the same `plot_anim.gnuplot` script visualizes the result.

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

## Run the example

From the repository root:

```sh
go run -C clients/go/cmd/example_batch . > clients/go/cmd/example_batch/output.txt
```

## Generate visualization

After running the example, generate an animated GIF with gnuplot:

```sh
cd clients/go/cmd/example_batch && gnuplot plot_anim.gnuplot
```

This creates `clients/go/cmd/example_batch/output.gif`.

The plot script is identical to the step-by-step example's. The headless recording
carries only vehicle trajectories, so per-tick traffic-light signals are not
replayed - the animation shows the scene and vehicles but no live signal state.

## What the example does

1. Creates a new simulation session
2. Pushes a grid of 30 cells forming 3 intersecting roads
3. Configures conflict zones at the intersection
4. Sets up a traffic light with 2 signal groups
5. Creates 3 vehicle trip generators (spawning cars, buses, and taxis)
6. Runs the full 50-tick horizon in a single `RunAndRecord` call and decodes the
   streamed columnar batches into per-tick vehicle rows

The output format is compatible with the gnuplot script for visualization.
