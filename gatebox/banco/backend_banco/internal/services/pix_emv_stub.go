package services

import (
	"crypto/sha256"
	"encoding/hex"
	"os"
	"regexp"
	"strconv"
	"strings"

	"banco_saczuck_backend/internal/gatebox_client"
)

// EMV BR Code PIX (copia e cola) começa com 000201; o Gatebox espera referências de charge próprias.
// Este stub valida localmente para sandbox / Developer Bank quando o QR é PIX EMV.
var emvTag54AmountRe = regexp.MustCompile(`54(\d{2})(\d+\.\d{2})`)

func pixEmvGateboxStub(reference string) (gatebox_client.ChargeValidationResponse, int, string, bool) {
	// Por padrão, preferimos validar via GateboxGo (/api/public/charges/validate), pois ele consegue
	// mapear o EMV -> txId (endToEndId) do QR gerado. O stub existe só como fallback manual.
	if strings.ToLower(strings.TrimSpace(os.Getenv("BANCO_PIX_EMV_VALIDATE_MODE"))) != "stub" {
		return gatebox_client.ChargeValidationResponse{}, 0, "", false
	}

	ref := strings.TrimSpace(reference)
	if len(ref) < 32 || !strings.HasPrefix(ref, "000201") {
		return gatebox_client.ChargeValidationResponse{}, 0, "", false
	}

	amountCents := int64(100) // R$ 1,00 se não achar valor no payload
	if m := emvTag54AmountRe.FindStringSubmatch(ref); len(m) == 3 {
		if f, err := strconv.ParseFloat(m[2], 64); err == nil && f >= 0.01 && f <= 100_000_000 {
			// Evita overflow / int4: máx. ~100M BRL em centavos cabe em int64; depois clamp no Pay.
			amountCents = int64(f*100 + 0.5)
		}
	}
	amountCents = clampAmountCentsForDB(amountCents)

	sum := sha256.Sum256([]byte(ref))
	chargeID := "sandbox-emv-" + hex.EncodeToString(sum[:10])

	return gatebox_client.ChargeValidationResponse{
		Valid:          true,
		ChargeID:       chargeID,
		AmountCents:    amountCents,
		Receiver:       "PIX_EMV_STUB",
		FailureMessage: "",
	}, 200, `{"stub":"br_code_pix_emv"}`, true
}
