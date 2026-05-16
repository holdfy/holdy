package gatebox_client

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

func truncateBytes(b []byte, max int) []byte {
	if len(b) <= max {
		return b
	}
	out := make([]byte, 0, max+32)
	out = append(out, b[:max]...)
	out = append(out, []byte(fmt.Sprintf("…[truncated,len=%d]", len(b)))...)
	return out
}

type Client struct {
	baseURL string
	apiKey  string
	http    *http.Client
}

func New(baseURL, apiKey string) *Client {
	return &Client{
		baseURL: baseURL,
		apiKey:  apiKey,
		http:    &http.Client{Timeout: 8 * time.Second},
	}
}

type ChargeValidationResponse struct {
	Valid          bool   `json:"valid"`
	ChargeID       string `json:"charge_id"`
	AmountCents    int64  `json:"amount_cents"`
	Receiver       string `json:"receiver"`
	FailureMessage string `json:"failure_message"`
}

func (c *Client) ValidateCharge(ctx context.Context, paymentRef string) (ChargeValidationResponse, int, string, error) {
	body := map[string]string{"reference": paymentRef}
	rawReq, _ := json.Marshal(body)
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost, c.baseURL+"/api/public/charges/validate", bytes.NewReader(rawReq))
	req.Header.Set("Authorization", "Bearer "+c.apiKey)
	req.Header.Set("Content-Type", "application/json")

	start := time.Now()
	resp, err := c.http.Do(req)
	if err != nil {
		return ChargeValidationResponse{}, 0, string(rawReq) + "|HTTP_ERROR:" + err.Error(), err
	}
	defer resp.Body.Close()

	bodyBytes, err := io.ReadAll(resp.Body)
	if err != nil {
		return ChargeValidationResponse{}, resp.StatusCode, string(rawReq) + "|READ_BODY:" + err.Error(), fmt.Errorf("read validate response: %w", err)
	}

	var out ChargeValidationResponse
	if err := json.Unmarshal(bodyBytes, &out); err != nil {
		return ChargeValidationResponse{}, resp.StatusCode,
			string(rawReq) + "|NON_JSON_BODY:" + string(truncateBytes(bodyBytes, 6000)),
			fmt.Errorf("decode validate response: %w", err)
	}
	rawResp, _ := json.Marshal(out)
	_ = start
	return out, resp.StatusCode, string(rawReq) + "|" + string(rawResp), nil
}

func (c *Client) NotifyStatus(ctx context.Context, paymentID, chargeID, status string) error {
	body := map[string]string{"payment_id": paymentID, "charge_id": chargeID, "status": status}
	rawReq, _ := json.Marshal(body)
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost, c.baseURL+"/api/internal/bank/notify-status", bytes.NewReader(rawReq))
	req.Header.Set("Authorization", "Bearer "+c.apiKey)
	req.Header.Set("Content-Type", "application/json")
	resp, err := c.http.Do(req)
	if err != nil {
		return err
	}
	resp.Body.Close()
	if resp.StatusCode >= 400 {
		return fmt.Errorf("notify status failed: %d", resp.StatusCode)
	}
	return nil
}

