#!/bin/bash
# Copia .env.example para .env (se .env não existir) e exibe variáveis principais.
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

if [ ! -f .env ]; then
  if [ -f .env.example ]; then
    cp .env.example .env
    echo "✅ .env criado a partir de .env.example"
  else
    echo "❌ .env.example não encontrado"
    exit 1
  fi
else
  echo "ℹ️  .env já existe"
fi

echo ""
echo "Variáveis (carregue com: source .env ou set -a && source .env && set +a):"
grep -E "^[A-Z_]+=" .env 2>/dev/null | sed 's/=.*/=***/' || true
echo ""
echo "Para subir só a API Rust: ./scripts/run-rust.sh"
echo "Para infra Docker unificada: ../../docker-compose.yml (na pasta money/) — ou ../../runinfra.sh / ../../runapp.sh start gatebox"
