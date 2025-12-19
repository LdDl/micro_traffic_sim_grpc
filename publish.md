## Regenerate clients (if not already done)
```bash
# Python
./scripts/gen_python.sh

# Go
./scripts/gen_go.sh
```

## Run tests
```bash
cargo test --features server
```


## Build release binaries
```bash
./scripts/build_release.sh
```

## Publish to crates.io
```bash
cargo publish
```

## Publish python client to PyPI
```bash
cd clients/python

# Activate venv
source .venv/bin/activate

# Install build tools
pip install build twine

# Build package
python -m build

# Upload to PyPI
twine upload dist/*

cd ../..
```

## Build and push Docker images
```bash
# Build
docker build -f Dockerfile.server -t micro-traffic-sim/server:latest .

# Tag for Docker Hub
docker tag micro-traffic-sim/server:latest dimahkiin/micro-traffic-sim-server:0.1.0
docker tag micro-traffic-sim/server:latest dimahkiin/micro-traffic-sim-server:latest

# Tag for GitHub Container Registry
docker tag micro-traffic-sim/server:latest ghcr.io/lddl/micro-traffic-sim-server:0.1.0
docker tag micro-traffic-sim/server:latest ghcr.io/lddl/micro-traffic-sim-server:latest

# Push to Docker Hub
docker push dimahkiin/micro-traffic-sim-server:0.1.0
docker push dimahkiin/micro-traffic-sim-server:latest

# Push to GitHub Container Registry
docker push ghcr.io/lddl/micro-traffic-sim-server:0.1.0
docker push ghcr.io/lddl/micro-traffic-sim-server:latest
```

## Git tags (+ golang client tag for pkg.go.dev)
```bash
# Main repo tag
git tag v0.1.0
git push origin v0.1.0

# Go submodule tag (for pkg.go.dev)
git tag clients/go/v0.1.0
git push origin clients/go/v0.1.0
```

## Verify releases

- **crates.io:** https://crates.io/crates/micro_traffic_sim
- **PyPI:** https://pypi.org/project/micro-traffic-sim/
- **pkg.go.dev:** https://pkg.go.dev/github.com/LdDl/micro_traffic_sim_grpc/clients/go@v0.1.0
- **Docker Hub:** https://hub.docker.com/r/dimahkiin/micro-traffic-sim-server/tags
- **GitHub Releases:** https://github.com/LdDl/micro_traffic_sim_grpc/releases
