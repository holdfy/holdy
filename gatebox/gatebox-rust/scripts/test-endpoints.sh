#!/bin/bash
# Testa endpoints do gatebox-rust (health, internal/metrics, swagger, 2112/metrics)
set -e

BASE_URL="${BASE_URL:-http://localhost:8080}"
METRICS_URL="${METRICS_URL:-http://localhost:2112}"

echo "=== Testando gatebox-rust ==="
echo ""

echo "1. Health (GET $BASE_URL/health):"
curl -s "$BASE_URL/health" | jq . 2>/dev/null || curl -s "$BASE_URL/health"
echo ""
echo ""

echo "2. Internal metrics JSON (GET $BASE_URL/internal/metrics):"
curl -s "$BASE_URL/internal/metrics" | jq . 2>/dev/null || curl -s "$BASE_URL/internal/metrics"
echo ""
echo ""

echo "3. Swagger com Basic Auth (GET $BASE_URL/swagger/ -u endpoint:segredo):"
HTTP=$(curl -s -w "\n%{http_code}" -u endpoint:segredo "$BASE_URL/swagger/")
CODE=$(echo "$HTTP" | tail -1)
BODY=$(echo "$HTTP" | head -n -1)
echo "HTTP $CODE"
echo "$BODY" | head -5
echo "..."
echo ""

echo "4. Prometheus metrics (GET $METRICS_URL/metrics):"
curl -s "$METRICS_URL/metrics" | head -15
echo "..."
echo ""

echo "=== Testes concluídos ==="
