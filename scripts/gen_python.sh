#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROTO_DIR="$ROOT_DIR/protos"
PYTHON_DIR="$ROOT_DIR/clients/python"
OUT_DIR="$PYTHON_DIR/micro_traffic_sim"
VENV_DIR="$PYTHON_DIR/.venv"

# Cleanup on exit or error
cleanup() {
  if [[ -n "${VIRTUAL_ENV:-}" ]]; then
    deactivate 2>/dev/null || true
  fi
}
trap cleanup EXIT

mkdir -p "$OUT_DIR"

# Check python3
if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 not found" >&2
  exit 1
fi

# Create venv if it doesn't exist
if [[ ! -d "$VENV_DIR" ]]; then
  echo "Creating virtual environment at $VENV_DIR..."
  python3 -m venv "$VENV_DIR"
fi

# Activate venv and install dependencies
echo "Activating virtual environment..."
source "$VENV_DIR/bin/activate"

echo "Installing dependencies..."
pip install -q -r "$PYTHON_DIR/requirements.txt"

# Generate Python code with type stubs (.pyi files)
echo "Generating Python gRPC client..."
python3 -m grpc_tools.protoc \
  -I"$PROTO_DIR" \
  --python_out="$OUT_DIR" \
  --pyi_out="$OUT_DIR" \
  --grpc_python_out="$OUT_DIR" \
  "$PROTO_DIR/service.proto" \
  "$PROTO_DIR/cell.proto" \
  "$PROTO_DIR/session.proto" \
  "$PROTO_DIR/step.proto" \
  "$PROTO_DIR/trip.proto" \
  "$PROTO_DIR/tls.proto" \
  "$PROTO_DIR/conflict_zones.proto" \
  "$PROTO_DIR/uuid.proto"

# Fix imports in generated files (change absolute to relative imports)
for f in "$OUT_DIR"/*_pb2*.py; do
  if [[ -f "$f" ]]; then
    # Replace "import xxx_pb2" with "from . import xxx_pb2"
    sed -i 's/^import \([a-z_]*_pb2\)/from . import \1/g' "$f"
  fi
done

# Install the package in editable mode
echo "Installing micro-traffic-sim package..."
pip install -q -e "$PYTHON_DIR"

echo ""
echo "Python client generated at $OUT_DIR"
echo "Virtual environment: $VENV_DIR"
echo ""
echo "To use:"
echo "  source $VENV_DIR/bin/activate"
echo "  python $PYTHON_DIR/examples/main.py"
