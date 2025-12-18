# Go client for micro_traffic_sim gRPC Server

[![Go Reference](https://pkg.go.dev/badge/github.com/LdDl/micro_traffic_sim_grpc/clients/go.svg)](https://pkg.go.dev/github.com/LdDl/micro_traffic_sim_grpc/clients/go)

Go client library for the microscopic traffic simulation gRPC server.

## Installation

```bash
go get github.com/LdDl/micro_traffic_sim_grpc/clients/go@latest
```

## Usage

```go
package main

import (
    "context"

    microtraffic "github.com/LdDl/micro_traffic_sim_grpc/clients/go"
    "google.golang.org/grpc"
    "google.golang.org/grpc/credentials/insecure"
)

func main() {
    conn, err := grpc.Dial("127.0.0.1:50051", grpc.WithTransportCredentials(insecure.NewCredentials()))
    if err != nil {
        panic(err)
    }
    defer conn.Close()

    client := microtraffic.NewServiceClient(conn)

    // Create a new session
    resp, err := client.NewSession(context.Background(), &microtraffic.SessionReq{Srid: 0})
    if err != nil {
        panic(err)
    }

    sessionID := resp.Id.Value
    // Use sessionID to push grid, trips, traffic lights, and run simulation...
}
```

## Documentation

- **API reference**: https://pkg.go.dev/github.com/LdDl/micro_traffic_sim_grpc/clients/go
- **Full example**: See [cmd/example/](cmd/example/) for a complete simulation workflow

## Running the example

1. Start the gRPC server:
```bash
cargo run --features server --bin micro_traffic_sim
```

2. Run the example (from repository root):
```bash
go run -C clients/go/cmd/example .
```

3. Generate visualization:
```bash
go run -C clients/go/cmd/example . > clients/go/cmd/example/output.txt
gnuplot clients/go/cmd/example/plot_anim.gnuplot
```
