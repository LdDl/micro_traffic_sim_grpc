# Python client for micro_traffic_sim gRPC Server

[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue.svg)](https://www.python.org/downloads/)
[![Typing: Typed](https://img.shields.io/badge/typing-typed-green.svg)](https://peps.python.org/pep-0561/)

Python client library for the microscopic traffic simulation gRPC server with full type hints support.

## Installation

```bash
pip install micro-traffic-sim
```

## Usage

```python
import grpc
from micro_traffic_sim import (
    ServiceStub,
    SessionReq,
    Cell,
    Point,
    ZoneType,
)

# Connect to server
channel = grpc.insecure_channel("127.0.0.1:50051")
client = ServiceStub(channel)

# Create a new session
response = client.NewSession(SessionReq(srid=0))
session_id = response.id.value

# Push grid cells, trips, traffic lights, and run simulation...
```

## Documentation

- **Full example**: See [examples/](https://github.com/LdDl/micro_traffic_sim_grpc/tree/master/clients/python/examples) for a complete simulation workflow

## Running the example

1. Start the gRPC server:
```bash
cargo run --features server --bin micro_traffic_sim
```

2. Run the example (from repository root):
```bash
source clients/python/.venv/bin/activate
python clients/python/examples/main.py
```

3. Generate visualization:
```bash
python clients/python/examples/main.py > clients/python/examples/output.txt
gnuplot clients/python/examples/plot_anim.gnuplot
```
