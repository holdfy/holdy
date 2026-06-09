#!/usr/bin/env bash
# Bootstrap único na pasta money/: cria `.env`, liga aos crates via symlink,
# opcionalmente sobe infra Docker (runinfra).
set -euo pipefail

MONEY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${MONEY}"

if [[ ! -f "${MONEY}/.env" ]]; then
  cp "${MONEY}/.env.example" "${MONEY}/.env"
  echo "==> Criado ${MONEY}/.env a partir de .env.example — edite segredos."
else
  echo "==> ${MONEY}/.env já existe."
fi

# Sincroniza todas as URLs de serviços locais com MONEY_LAN_HOST.
# Resolve o problema de IP hardcoded quando se muda de máquina ou de rede:
# basta atualizar MONEY_LAN_HOST no .env e reexecutar este script.
_sync_service_ips() {
  python3 - "${MONEY}/.env" <<'PYEOF'
import sys, re

envfile = sys.argv[1]
ip_re = re.compile(r'\b(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})\b')

with open(envfile) as f:
    lines = f.readlines()

host = None
for line in lines:
    if line.startswith('MONEY_LAN_HOST='):
        host = line.split('=', 1)[1].strip()
        break

if not host:
    sys.exit(0)

# Variáveis cujo IP deve seguir MONEY_LAN_HOST (serviços Docker locais)
SYNC_VARS = {
    'DATABASE_URL', 'REDIS_URL', 'NATS_URL',
    'PULSAR_URL', 'PULSAR_SERVICE_URL', 'PULSAR_ADMIN_URL',
    'GATEBOX_BASE_URL', 'APICASH_CORE_URL',
    'PUBLIC_APP_URL', 'ADMIN_API_URL', 'APICASH_ADMIN_API_URL',
    'APICASH_HTTP_BASE', 'APICASH_ADMIN_HTTP_BASE',
    'BANCO_DATABASE_URL', 'BANCO_WEBHOOK_URL',
    'POSTGRESQL_WRITE_URL', 'POSTGRESQL_READ_URL',
    'SULCRED_OUT_URL', 'SCRAPER_URL',
    'APICASH_TRACKING_SIMULATOR_URL', 'LOGISTICA_WHATSAPP_NOTIFY_URL',
}

changed = []
out = []
for line in lines:
    key = line.split('=', 1)[0].strip()
    if key in SYNC_VARS and not line.startswith('#'):
        m = ip_re.search(line)
        if m and m.group(1) != host:
            old_ip = m.group(1)
            new_line = line.replace(old_ip, host)
            out.append(new_line)
            changed.append(f"  {key}: {old_ip} → {host}")
            continue
    out.append(line)

if changed:
    with open(envfile, 'w') as f:
        f.writelines(out)
    print(f"==> IPs de serviços locais sincronizados com MONEY_LAN_HOST={host}:")
    for c in changed:
        print(c)
PYEOF
}
_sync_service_ips

# Templates antigos: `KEY=valor com espaços` sem aspas faz o bash tratar a 2.ª palavra como comando ao fazer source.
if grep -q '^GATEBOX_CUSTOMER_NAME=APICash Platform$' "${MONEY}/.env" 2>/dev/null; then
  sed -i 's/^GATEBOX_CUSTOMER_NAME=APICash Platform$/GATEBOX_CUSTOMER_NAME="HoldFy Platform"/' "${MONEY}/.env"
  echo "==> Corrigido GATEBOX_CUSTOMER_NAME no .env (aspas para valores com espaços + nome HoldFy)."
fi

if grep -q '^GATEBOX_CUSTOMER_NAME="APICash Platform"$' "${MONEY}/.env" 2>/dev/null; then
  sed -i 's/^GATEBOX_CUSTOMER_NAME="APICash Platform"$/GATEBOX_CUSTOMER_NAME="HoldFy Platform"/' "${MONEY}/.env"
  echo "==> Atualizado GATEBOX_CUSTOMER_NAME para HoldFy Platform."
fi

# Postgres Gatebox unificado no serviço `postgres` (porta 5432, user POSTGRES_*).
if grep -qE '5433/dubai-cash|GATEBOX_POSTGRES_PORT=5433' "${MONEY}/.env" 2>/dev/null; then
  sed -i '/^GATEBOX_POSTGRES_PORT=/d; /^GATEBOX_POSTGRES_USER=/d; /^GATEBOX_POSTGRES_PASSWORD=/d; /^# GATEBOX_DB_HOST_MOUNT=/d' "${MONEY}/.env"
  sed -i 's|^# Compose money: gatebox-postgres.*|# Compose money: Postgres único (5432). Database Gatebox: dubai-cash.|' "${MONEY}/.env"
  lan_host="$(grep -E '^MONEY_LAN_HOST=' "${MONEY}/.env" 2>/dev/null | head -1 | cut -d= -f2- || true)"
  pguser="$(grep -E '^POSTGRES_USER=' "${MONEY}/.env" 2>/dev/null | head -1 | cut -d= -f2- || echo apicash)"
  pgpass="$(grep -E '^POSTGRES_PASSWORD=' "${MONEY}/.env" 2>/dev/null | head -1 | cut -d= -f2- || echo apicash)"
  pgport="$(grep -E '^POSTGRES_PORT=' "${MONEY}/.env" 2>/dev/null | head -1 | cut -d= -f2- || echo 5432)"
  [[ -z "${lan_host}" ]] && lan_host="127.0.0.1"
  sed -i "s|^POSTGRESQL_WRITE_URL=.*|POSTGRESQL_WRITE_URL=postgres://${pguser}:${pgpass}@${lan_host}:${pgport}/dubai-cash?sslmode=disable|" "${MONEY}/.env"
  sed -i "s|^POSTGRESQL_READ_URL=.*|POSTGRESQL_READ_URL=postgres://${pguser}:${pgpass}@${lan_host}:${pgport}/dubai-cash?sslmode=disable|" "${MONEY}/.env"
  echo "==> Atualizado Gatebox DB para Postgres único (${lan_host}:${pgport}/dubai-cash)."
fi

# Um único .env em money/; symlinks para crates que leem só no CWD.
ln -sfn "${MONEY}/.env" "${MONEY}/apicash/.env"

if [[ "${MONEY}" == "/home/devel/git/pos-nearx/money" ]] && [[ ! -f "${MONEY}/../gatebox/gatebox-rust/Cargo.toml" ]] && [[ ! -f "/home/devel/git/pos-nearx/gatebox/gatebox-rust/Cargo.toml" ]] && [[ "${SKIP_GATEBOX_REPO_HINT:-0}" != "1" ]]; then
  echo "[setup-env][hint] Esperado Gatebox fonte em /home/devel/git/pos-nearx/gatebox (pastas gatebox-rust/, simulador_rust/)."
fi

# Gatebox: symlink ../gatebox quando não há crate Rust válido dentro de money/gatebox/.
if [[ -f "${MONEY}/../gatebox/gatebox-rust/Cargo.toml" ]]; then
  if [[ ! -e "${MONEY}/gatebox" ]] || [[ -L "${MONEY}/gatebox" ]]; then
    ln -sfn "../gatebox" "${MONEY}/gatebox"
    echo "==> Symlink ${MONEY}/gatebox → ../gatebox"
  elif [[ -d "${MONEY}/gatebox/gatebox-rust" ]] && [[ ! -f "${MONEY}/gatebox/gatebox-rust/Cargo.toml" ]]; then
    echo "[setup-env][warn] Existe ${MONEY}/gatebox/gatebox-rust/ sem Cargo.toml (muitas vezes só db/ pelo Docker Compose)."
    echo "           Para ./runapp usar o código em ../gatebox: apagar a pasta inteira (pode precisar sudo) e rerun:"
    echo "           sudo rm -rf \"${MONEY}/gatebox\" && \"$0\""
    echo "           Ou GATEBOX_RUST_DIR=$MONEY/../gatebox/gatebox-rust e no .env GATEBOX_DB_HOST_MOUNT=$MONEY/../gatebox/gatebox-rust/db"
  fi
elif [[ -d "${MONEY}/gatebox/gatebox-rust" ]] && [[ ! -f "${MONEY}/gatebox/gatebox-rust/Cargo.toml" ]] && [[ ! -f "${MONEY}/../gatebox/gatebox-rust/Cargo.toml" ]]; then
  echo "[setup-env][warn] money/gatebox/gatebox-rust/ sem Cargo.toml e sem ../gatebox clone — defina GATEBOX_RUST_DIR ou adicione o repositório gatebox."
fi

# Gatebox (opcional — só criar symlink se o diretório existir de novo no tree):
if [[ -d "${MONEY}/gatebox/gatebox-rust" ]]; then
  ln -sfn "${MONEY}/.env" "${MONEY}/gatebox/gatebox-rust/.env"
  echo "==> Symlink gatebox/gatebox-rust/.env → money/.env"
fi
if [[ -d "${MONEY}/gatebox/banco/backend_banco" ]]; then
  ln -sfn "${MONEY}/.env" "${MONEY}/gatebox/banco/backend_banco/.env"
  echo "==> Symlink gatebox/banco/backend_banco/.env → money/.env"
fi

echo "==> Symlink apicash/.env → money/.env (gatebox só se pastas existirem)"

if [[ "${SKIP_INFRA:-0}" != "1" ]] && [[ -x "${MONEY}/runinfra.sh" ]]; then
  echo "==> Subindo infra (Docker + migrações)…"
  "${MONEY}/runinfra.sh" start
else
  if [[ "${SKIP_INFRA:-0}" = "1" ]]; then
    echo "==> SKIP_INFRA=1 — não chamando runinfra.sh"
  fi
fi

echo ""
echo "==> OK. Apps: ${MONEY}/runapp.sh   | Estado infra: ${MONEY}/runinfra.sh status"
echo "    Gatebox+WhatsApp: alinhe money/.env com .env.example (APICash: GATEBOX_BASE_URL; Gatebox: gateways PIX);"
echo "    depois: ${MONEY}/runapp.sh start  e  ${MONEY}/scripts/verify-money-stack.sh"
