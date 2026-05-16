#!/bin/bash
# Sobe a API Gatebox Rust (cargo run). Carrega .env se existir.
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

if [ -f .env ]; then
  # .env deve preencher defaults, mas não sobrescrever variáveis já exportadas pelo money/runapp.sh.
  # Preserva vars críticas se já estiverem setadas no ambiente.
  _preserve_vars=(PORT METRICS_PORT ENABLE_METRICS POSTGRESQL_WRITE_URL POSTGRESQL_READ_URL MESSAGING_BACKEND PULSAR_URL RBMQ_URI)
  for _v in "${_preserve_vars[@]}"; do
    eval "_saved_${_v}=\"\${${_v}:-}\""
  done

  set -a
  source .env
  set +a

  for _v in "${_preserve_vars[@]}"; do
    eval "_sv=\"\${_saved_${_v}:-}\""
    if [ -n "${_sv}" ]; then
      export "${_v}=${_sv}"
    fi
  done
  echo "📦 Variáveis carregadas de .env (sem sobrescrever env já setado)"
fi

PORT="${PORT:-8080}"
echo "🚀 Iniciando gatebox-rust (PORT=$PORT)..."
if [ "${SKIP_BUILD:-0}" != "1" ]; then
  cargo build --release
fi
exec ./target/release/gatebox-rust
