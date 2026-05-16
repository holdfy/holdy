package services

import (
	"encoding/json"
	"fmt"
	"strings"

	"banco_saczuck_backend/internal/gatebox_client"
)

const maxLogFieldRunes = 12000

func truncateForLog(s string, maxRunes int) string {
	s = strings.TrimSpace(s)
	r := []rune(s)
	if len(r) <= maxRunes {
		return s
	}
	return string(r[:maxRunes]) + fmt.Sprintf("…[truncated,rune_len=%d]", len(r))
}

// referenceLogSummary descreve o QR/referência sem inundar logs (cabeçalho + cauda).
func referenceLogSummary(ref string) string {
	ref = strings.TrimSpace(ref)
	if ref == "" {
		return "empty"
	}
	r := []rune(ref)
	n := len(r)
	headN := 160
	if headN > n {
		headN = n
	}
	head := string(r[:headN])
	tail := ""
	if n > 200 {
		tailN := 120
		if tailN > n {
			tailN = n
		}
		tail = string(r[n-tailN:])
	}
	return fmt.Sprintf("byte_len=%d head=%q tail=%q", len(ref), head, tail)
}

func formatValidationForLog(sc int, v gatebox_client.ChargeValidationResponse, exchange string) string {
	meta, _ := json.Marshal(map[string]any{
		"http_status":      sc,
		"valid":            v.Valid,
		"failure_message":  v.FailureMessage,
		"charge_id":        v.ChargeID,
		"amount_cents":     v.AmountCents,
		"receiver":         v.Receiver,
		"exchange_summary": truncateForLog(exchange, maxLogFieldRunes),
	})
	return string(meta)
}
