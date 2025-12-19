#!/bin/bash
# Build release binaries for Linux and Windows
#
# Usage:
#   ./scripts/build_release.sh
#
# Output:
#   dist/micro-traffic-sim-server-linux-amd64.tar.gz
#   dist/micro-traffic-sim-server-windows-amd64.zip (if cross-compile available)

set -e

VERSION="${VERSION:-$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')}"
DIST_DIR="dist"
BINARY_NAME="micro_traffic_sim"

echo "Building micro-traffic-sim-server v${VERSION}"
mkdir -p "${DIST_DIR}"

# Build Linux (native)
echo ""
echo "=== Building Linux amd64 ==="
cargo build --release --features server

LINUX_ARCHIVE="micro-traffic-sim-server-${VERSION}-linux-amd64.tar.gz"
cp "target/release/${BINARY_NAME}" "${DIST_DIR}/${BINARY_NAME}"
tar -czvf "${DIST_DIR}/${LINUX_ARCHIVE}" -C "${DIST_DIR}" "${BINARY_NAME}"
rm "${DIST_DIR}/${BINARY_NAME}"
echo "Created: ${DIST_DIR}/${LINUX_ARCHIVE}"

# Build Windows (cross-compile if toolchain available)
# @todo: could not work in some scenrios...Need to bmake it better
echo ""
echo "=== Building Windows amd64 ==="

WINDOWS_TARGET="x86_64-pc-windows-gnu"

if rustup target list --installed | grep -q "${WINDOWS_TARGET}"; then
    if command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        cargo build --release --features server --target "${WINDOWS_TARGET}"

        WINDOWS_ARCHIVE="micro-traffic-sim-server-${VERSION}-windows-amd64.zip"
        cp "target/${WINDOWS_TARGET}/release/${BINARY_NAME}.exe" "${DIST_DIR}/${BINARY_NAME}.exe"
        (cd "${DIST_DIR}" && zip "${WINDOWS_ARCHIVE}" "${BINARY_NAME}.exe")
        rm "${DIST_DIR}/${BINARY_NAME}.exe"
        echo "Created: ${DIST_DIR}/${WINDOWS_ARCHIVE}"
    else
        echo "SKIP: mingw-w64 toolchain not installed"
        echo "  Install with: sudo pacman -S mingw-w64-gcc (Arch) or apt install mingw-w64 (Debian)"
    fi
else
    echo "SKIP: Rust target ${WINDOWS_TARGET} not installed"
    echo "  Install with: rustup target add ${WINDOWS_TARGET}"
fi

echo ""
echo "=== Done ==="
ls -la "${DIST_DIR}/"
