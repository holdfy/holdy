#!/bin/bash
# Sobe os três simuladores em background (seventrust, sulcred, client-simulator)
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

GATEWAY_URL="${GATEWAY_URL:-http://localhost:8080}"
GATEBOXGO_WEBHOOK_URL="${GATEBOXGO_WEBHOOK_URL:-http://localhost:8080}"

echo "Iniciando simuladores..."
echo "  GATEWAY_URL=$GATEWAY_URL"
echo "  GATEBOXGO_WEBHOOK_URL=$GATEBOXGO_WEBHOOK_URL"

PORT=7010 GATEBOXGO_WEBHOOK_URL="$GATEBOXGO_WEBHOOK_URL" cargo run -p seventrust &
SEVENTRUST_PID=$!
sleep 1

PORT=7020 GATEBOXGO_WEBHOOK_URL="$GATEBOXGO_WEBHOOK_URL" cargo run -p sulcred &
SULCRED_PID=$!
sleep 1

PORT=7070 GATEWAY_URL="$GATEWAY_URL" cargo run -p client-simulator &
CLIENT_PID=$!

echo ""
echo "Simuladores iniciados:"
echo "  seventrust (7010): PID $SEVENTRUST_PID"
echo "  sulcred (7020):    PID $SULCRED_PID"
echo "  client-simulator (7070): PID $CLIENT_PID"
echo ""
echo "Health: curl http://localhost:7010/health"
echo "Health: curl http://localhost:7020/health"
echo "Health: curl http://localhost:7070/health"
echo "Swagger: http://localhost:7070/swagger/"
echo ""
echo "Para parar: kill $SEVENTRUST_PID $SULCRED_PID $CLIENT_PID"
