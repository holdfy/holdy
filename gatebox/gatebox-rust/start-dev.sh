#!/bin/bash
#
# Compat: "start-dev" dentro do gatebox-rust.
# Sobe SOMENTE a infra necessária (Docker) para rodar a API Rust + simuladores Rust no host.
#
# Para subir tudo (infra + API + simuladores): ../../runapp.sh start gatebox
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║        🚀 Iniciando Infra (Gatebox Rust)                      ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

bash ./scripts/start-infra.sh


echo ""
echo "Próximos passos (Rust-only):"
echo "  - Subir API + simuladores: ../../runapp.sh start gatebox"
echo "  - Ver status:             ../../runapp.sh status gatebox"
echo "  - Parar processos host:   ../../runapp.sh stop gatebox"
echo ""

