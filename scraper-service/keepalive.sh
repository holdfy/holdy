#!/usr/bin/env bash
# Cron diário: mantém a sessão TikTok ativa renovando os cookies.
# Se TT_EMAIL + TT_PASSWORD estiverem no .env, faz re-login automático
# quando a sessão expirar.
#
# Crontab: 0 4 * * * /home/holdfy/git/holdy/scraper-service/keepalive.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MONEY_ENV="$(dirname "${SCRIPT_DIR}")/money/.env"

# Carregar variáveis de ambiente do projeto
if [ -f "${MONEY_ENV}" ]; then
  set -a
  # shellcheck disable=SC1090
  . "${MONEY_ENV}"
  set +a
fi

PYTHON="${SCRIPT_DIR}/.venv/bin/python"

if [ ! -x "${PYTHON}" ]; then
  echo "[keepalive] venv não encontrado — execute setup primeiro" >&2
  exit 1
fi

echo "[keepalive] $(date '+%Y-%m-%d %H:%M:%S') iniciando"
"${PYTHON}" "${SCRIPT_DIR}/scraper.py" keepalive
echo "[keepalive] $(date '+%Y-%m-%d %H:%M:%S') concluído"
