#!/usr/bin/env bash
# acerta_ip.sh — descobre o IP LAN desta máquina e atualiza todas as configs dos apps.
#
# Uso:
#   ./acerta_ip.sh              # detecta o IP automaticamente
#   ./acerta_ip.sh 192.168.1.5  # usa o IP informado
#
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MONEY="${REPO_ROOT}/money"
ENV_FILE="${MONEY}/.env"
ENV_EXAMPLE="${MONEY}/.env.example"
BANCO_DART="${REPO_ROOT}/gatebox/banco/app_banco/lib/src/core/banco_api_config.dart"

# ── Cores ────────────────────────────────────────────────────────────────────
GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; RESET='\033[0m'
ok()   { echo -e "${GREEN}  [ok]${RESET}  $*"; }
skip() { echo -e "${YELLOW}  [--]${RESET}  $*"; }
err()  { echo -e "${RED}  [!!]${RESET}  $*"; }

# ── Detectar IP LAN ──────────────────────────────────────────────────────────
detect_lan_ip() {
  local ip

  # 1. ip route get: devolve o src da rota para sair à internet
  ip=$(ip route get 1.1.1.1 2>/dev/null \
       | awk '/src/ { for(i=1;i<=NF;i++) if($i=="src"){ print $(i+1); exit } }')
  if [[ -n "${ip:-}" && "${ip}" != "127."* ]]; then
    echo "${ip}"; return
  fi

  # 2. Primeira interface global que não seja loopback/emulador Android
  ip=$(ip -4 addr show scope global 2>/dev/null \
       | awk '/inet/ { split($2,a,"/"); print a[1] }' \
       | grep -v "^10\.0\.2\." | head -1)
  if [[ -n "${ip:-}" ]]; then
    echo "${ip}"; return
  fi

  # 3. hostname -I (último recurso)
  hostname -I 2>/dev/null | awk '{print $1}'
}

# ── Substituição segura em arquivo ───────────────────────────────────────────
# Uso: replace_line FILE SED_PATTERN SED_REPLACEMENT DESCRICAO
replace_line() {
  local file="$1" pattern="$2" repl="$3" desc="${4:-${file}}"
  if [[ ! -f "${file}" ]]; then
    skip "${desc}: arquivo não encontrado"
    return
  fi
  if grep -qE "${pattern}" "${file}" 2>/dev/null; then
    local before after
    before=$(grep -E "${pattern}" "${file}" | head -1)
    sed -i -E "s|${pattern}|${repl}|g" "${file}"
    after=$(grep -E "${pattern//[0-9]./[0-9].}" "${file}" 2>/dev/null | head -1 || \
            grep -F "${NEW_IP}" "${file}" | grep -E "$(echo "${repl}" | sed 's/\\\(.*\\\)/.*/g')" | head -1 || true)
    ok "${desc}"
    echo "         antes:  ${before}"
    echo "         depois: ${after:-$(grep -F "${NEW_IP}" "${file}" | head -1)}"
  else
    skip "${desc}: padrão não encontrado (já atualizado?)"
  fi
}

# ── IP ───────────────────────────────────────────────────────────────────────
if [[ "${1:-}" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  NEW_IP="$1"
  echo -e "\nIP fornecido: ${GREEN}${NEW_IP}${RESET}"
else
  NEW_IP="$(detect_lan_ip)"
  if [[ -z "${NEW_IP:-}" ]]; then
    err "Não foi possível detectar o IP LAN."
    echo "  Passe o IP manualmente:  ./acerta_ip.sh 192.168.x.x"
    exit 1
  fi
  echo -e "\nIP detectado: ${GREEN}${NEW_IP}${RESET}"
fi
echo ""

# ═════════════════════════════════════════════════════════════════════════════
# 1. money/.env  →  MONEY_LAN_HOST
# ═════════════════════════════════════════════════════════════════════════════
echo "── money/.env ───────────────────────────────────────────"
replace_line \
  "${ENV_FILE}" \
  "^(MONEY_LAN_HOST=).*" \
  "\1${NEW_IP}" \
  "MONEY_LAN_HOST"

# ═════════════════════════════════════════════════════════════════════════════
# 2. money/.env.example  →  todas as ocorrências de IPs LAN (não localhost/127/10.0.2)
# ═════════════════════════════════════════════════════════════════════════════
echo ""
echo "── money/.env.example ───────────────────────────────────"
if [[ -f "${ENV_EXAMPLE}" ]]; then
  # Substitui qualquer 192.168.x.x que não seja em comentário
  CHANGED=0
  while IFS= read -r line; do
    if echo "${line}" | grep -qE "^[^#].*192\.[0-9]+\.[0-9]+\.[0-9]+"; then
      OLD_IP=$(echo "${line}" | grep -oE "192\.[0-9]+\.[0-9]+\.[0-9]+" | head -1)
      if [[ "${OLD_IP}" != "${NEW_IP}" ]]; then
        ok ".env.example: ${OLD_IP} → ${NEW_IP}  (${line%%=*})"
        CHANGED=1
      fi
    fi
  done < "${ENV_EXAMPLE}"
  if [[ "${CHANGED}" -eq 1 ]]; then
    sed -i -E "s|192\.[0-9]+\.[0-9]+\.[0-9]+|${NEW_IP}|g" "${ENV_EXAMPLE}"
  else
    skip ".env.example: nenhum IP LAN a substituir (já é ${NEW_IP})"
  fi
else
  skip ".env.example não encontrado"
fi

# ═════════════════════════════════════════════════════════════════════════════
# 3. app_banco Flutter  →  defaultLanHost (BANCO_API_LAN_HOST)
# ═════════════════════════════════════════════════════════════════════════════
echo ""
echo "── app_banco (Flutter) ──────────────────────────────────"
if [[ -f "${BANCO_DART}" ]]; then
  # A linha em questão é: defaultValue: '192.168.x.x',
  # Só substituímos IPs dentro de aspas simples (não toca comentários)
  if grep -qE "defaultValue: '[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+'" "${BANCO_DART}"; then
    before=$(grep -E "defaultValue: '[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+'" "${BANCO_DART}" | head -1)
    sed -i -E "s|(defaultValue: ')[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+'|\1${NEW_IP}'|g" "${BANCO_DART}"
    after=$(grep -E "defaultValue: '${NEW_IP//./\\.}'" "${BANCO_DART}" | head -1 || true)
    ok "banco_api_config.dart"
    echo "         antes:  ${before}"
    echo "         depois: ${after}"
  else
    skip "banco_api_config.dart: IP já é ${NEW_IP} ou padrão não encontrado"
  fi
else
  skip "banco_api_config.dart não encontrado em ${BANCO_DART}"
fi

# ═════════════════════════════════════════════════════════════════════════════
# Resumo
# ═════════════════════════════════════════════════════════════════════════════
echo ""
echo -e "══════════════════════════════════════════════════════════"
echo -e "  IP configurado: ${GREEN}${NEW_IP}${RESET}"
echo ""
echo "  Próximos passos:"
echo "    cd money && ./runapp.sh restart all    # recarrega .env nos serviços"
echo "    flutter run (app_banco)                # rebuild com novo IP embutido"
echo -e "══════════════════════════════════════════════════════════"
