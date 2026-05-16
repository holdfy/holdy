#!/bin/bash
# Build das imagens Docker dos simuladores Rust
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

echo "Building simulador_rust Docker images..."

docker build --build-arg PACKAGE=seventrust -t seventrust-rust .
docker build --build-arg PACKAGE=sulcred -t sulcred-rust .
docker build --build-arg PACKAGE=client-simulator -t client-simulator-rust .

echo ""
echo "Imagens criadas:"
echo "  seventrust-rust"
echo "  sulcred-rust"
echo "  client-simulator-rust"
echo ""
echo "Para usar com o stack único deste repo (infra + apps na pasta money/):"
echo "  cd ../.."
echo "  ./runinfra.sh"
echo "  ./runapp.sh start gatebox"

