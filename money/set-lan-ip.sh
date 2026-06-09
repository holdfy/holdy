#!/usr/bin/env bash
# set-lan-ip.sh — detecta o IP LAN da máquina e actualiza todos os serviços do monorepo.
#
# Uso:
#   ./set-lan-ip.sh            # auto-detecta o IP da interface de rede activa
#   ./set-lan-ip.sh 192.168.x.x  # usa o IP passado manualmente
#
# O que actualiza:
#   - money/.env : MONEY_LAN_HOST + todas as URLs com 127.0.0.1 / localhost
#   - apprastreio backend .env (se existir)
#   - apprastreio/app api_config.dart : defaultLanHost
#   - gatebox/banco app banco_api_config.dart : defaultLanHost
#   - gatebox/banco backend .env (se existir)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "${SCRIPT_DIR}/.." && pwd)"
MONEY="${SCRIPT_DIR}"

# ── 1. Detectar / validar IP ────────────────────────────────────────────────
if [[ -n "${1:-}" ]]; then
  LAN_IP="$1"
  echo "==> IP fornecido manualmente: ${LAN_IP}"
else
  # ip route get: retorna o src usado para chegar a 1.1.1.1
  LAN_IP="$(ip route get 1.1.1.1 2>/dev/null | grep -oP 'src \K[^\s]+' | head -1 || true)"
  if [[ -z "${LAN_IP}" ]]; then
    # fallback: primeiro IP não-loopback
    LAN_IP="$(hostname -I 2>/dev/null | awk '{print $1}' || true)"
  fi
  if [[ -z "${LAN_IP}" ]] || [[ "${LAN_IP}" == "127.0.0.1" ]]; then
    echo "ERRO: Não foi possível detectar o IP LAN automaticamente."
    echo "      Passe como argumento: $0 192.168.x.x"
    exit 1
  fi
  echo "==> IP LAN detectado automaticamente: ${LAN_IP}"
fi

# Validação básica de formato IPv4
if ! [[ "${LAN_IP}" =~ ^[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}$ ]]; then
  echo "ERRO: '${LAN_IP}' não parece um IPv4 válido."
  exit 1
fi

echo ""

# ── 2. money/.env ────────────────────────────────────────────────────────────
update_env_file() {
  local env_file="$1"
  local label="$2"

  if [[ ! -f "${env_file}" ]]; then
    echo "  [skip] ${label} não existe"
    return
  fi

  local changed=0
  local old_lan=""

  # MONEY_LAN_HOST — lê o valor ANTIGO antes de sobrescrever
  if grep -q '^MONEY_LAN_HOST=' "${env_file}"; then
    old_lan="$(grep '^MONEY_LAN_HOST=' "${env_file}" | head -1 | cut -d= -f2-)"
    if [[ "${old_lan}" != "${LAN_IP}" ]]; then
      sed -i "s|^MONEY_LAN_HOST=.*|MONEY_LAN_HOST=${LAN_IP}|" "${env_file}"
      echo "  ${label}: MONEY_LAN_HOST ${old_lan} → ${LAN_IP}"
      changed=1
    fi
  fi

  # Substitui o IP antigo do MONEY_LAN_HOST em TODAS as linhas que não são comentário.
  # Isso cobre a troca de máquina/rede: se o IP era 10.20.3.75 e agora é 192.168.1.10,
  # qualquer URL que ainda usasse o IP antigo é corrigida aqui.
  if [[ -n "${old_lan}" && "${old_lan}" != "${LAN_IP}" \
        && "${old_lan}" != "127.0.0.1" && "${old_lan}" != "localhost" ]]; then
    if grep -v '^#' "${env_file}" | grep -qF "${old_lan}"; then
      sed -i "/^[^#]/s|${old_lan}|${LAN_IP}|g" "${env_file}"
      echo "  ${label}: ${old_lan} → ${LAN_IP} (IP anterior encontrado nas URLs)"
      changed=1
    fi
  fi

  # Substitui 127.0.0.1 e localhost em valores de URL (ignora linhas de comentário)
  # Esquemas abrangidos: http://, https://, ws://, wss://, redis://, pulsar://, postgres://, mongodb://
  local url_pattern='https\?://\|wss\?://\|redis://\|pulsar://\|postgres://\|mongodb://'

  # 127.0.0.1
  if grep -v '^#' "${env_file}" | grep -qE "(${url_pattern//\\|/|})127\.0\.0\.1"; then
    sed -i "/^[^#]/s|://127\.0\.0\.1|://${LAN_IP}|g" "${env_file}"
    echo "  ${label}: 127.0.0.1 → ${LAN_IP} (URLs)"
    changed=1
  fi

  # localhost
  if grep -v '^#' "${env_file}" | grep -qE "(${url_pattern//\\|/|})localhost"; then
    sed -i "/^[^#]/s|://localhost|://${LAN_IP}|g" "${env_file}"
    echo "  ${label}: localhost → ${LAN_IP} (URLs)"
    changed=1
  fi

  # Substitui qualquer IP privado RFC 1918 restante nas linhas que não são comentário.
  # Apanha IPs obsoletos de máquinas ou redes anteriores que não eram 127.0.0.1 nem o old_lan.
  while IFS= read -r stale_ip; do
    [[ "${stale_ip}" == "${LAN_IP}" ]] && continue
    sed -i "/^[^#]/s|${stale_ip}|${LAN_IP}|g" "${env_file}"
    echo "  ${label}: ${stale_ip} → ${LAN_IP} (IP privado obsoleto)"
    changed=1
  done < <(grep -v '^#' "${env_file}" \
    | grep -oE '\b(10\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}|172\.(1[6-9]|2[0-9]|3[01])\.[0-9]{1,3}\.[0-9]{1,3}|192\.168\.[0-9]{1,3}\.[0-9]{1,3})\b' \
    | sort -u)

  if [[ ${changed} -eq 0 ]]; then
    echo "  ${label}: já actualizado (sem alterações)"
  fi
}

update_env_file "${MONEY}/.env"                                            "money/.env"
update_env_file "${REPO}/apprastreio/backend/.env"                        "apprastreio/backend/.env"
update_env_file "${REPO}/gatebox/banco/backend_banco/.env"                "banco/backend_banco/.env"

# ── 3. Dart — actualiza defaultLanHost nos configs de API ────────────────────
update_dart_lan_host() {
  local dart_file="$1"
  local label="$2"

  if [[ ! -f "${dart_file}" ]]; then
    echo "  [skip] ${label} não existe"
    return
  fi

  # Extrai o IP actualmente em defaultValue: 'x.x.x.x' (único por ficheiro)
  local old_ip
  old_ip="$(grep -oP "defaultValue: '\K[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+" "${dart_file}" | head -1 || true)"

  if [[ -z "${old_ip}" ]]; then
    echo "  [skip] ${label}: nenhum IP em defaultValue encontrado"
    return
  fi

  if [[ "${old_ip}" == "${LAN_IP}" ]]; then
    echo "  ${label}: já ${LAN_IP} (sem alteração)"
    return
  fi

  sed -i "s|defaultValue: '${old_ip}'|defaultValue: '${LAN_IP}'|g" "${dart_file}"
  echo "  ${label}: defaultLanHost ${old_ip} → ${LAN_IP}"
}

echo ""
update_dart_lan_host \
  "${REPO}/apprastreio/app/lib/src/core/api_config.dart" \
  "apprastreio api_config.dart"

update_dart_lan_host \
  "${REPO}/gatebox/banco/app_banco/lib/src/core/banco_api_config.dart" \
  "banco_api_config.dart"

# ── 4. Resumo e comandos flutter run ─────────────────────────────────────────
echo ""
echo "══════════════════════════════════════════════════════════"
echo " IP LAN configurado: ${LAN_IP}"
echo "══════════════════════════════════════════════════════════"
echo ""
echo "Reinicie os serviços backend para o .env ter efeito:"
echo "  cd money && ./runapp.sh restart all"
echo ""
echo "Flutter rastreio (telefone físico):"
echo "  cd apprastreio/app && flutter run \\"
echo "    --dart-define=LOGISTICA_API_LAN_HOST=${LAN_IP}"
echo ""
echo "Flutter banco (telefone físico):"
echo "  cd gatebox/banco/app_banco && flutter run \\"
echo "    --dart-define=BANCO_API_LAN_HOST=${LAN_IP}"
echo ""
