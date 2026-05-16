package httpapi

import (
	"database/sql"
	"log"
	"net/http"
	"time"

	"banco_saczuck_backend/internal/config"
	"banco_saczuck_backend/internal/controllers"
	"banco_saczuck_backend/internal/gatebox_client"
	"banco_saczuck_backend/internal/models"
	"banco_saczuck_backend/internal/repositories"
	"banco_saczuck_backend/internal/services"
)

func NewRouter(cfg config.Config, db *sql.DB, logger *log.Logger) http.Handler {
	repo := repositories.New(db)
	gatebox := gatebox_client.New(cfg.GateboxBaseURL, cfg.GateboxAPIKey)
	svc := services.NewBankService(repo, gatebox, logger)
	ctrl := controllers.New(svc)

	mux := http.NewServeMux()
	mux.HandleFunc("GET /health", ctrl.Health)
	mux.HandleFunc("POST /auth/login", ctrl.Login)
	mux.HandleFunc("POST /accounts", ctrl.CreateAccount)
	mux.HandleFunc("GET /accounts/me", ctrl.GetMe)
	mux.HandleFunc("GET /accounts/me/balance", ctrl.GetBalance)
	mux.HandleFunc("POST /accounts/me/topup", ctrl.Topup)
	mux.HandleFunc("GET /transactions", ctrl.ListTransactions)
	mux.HandleFunc("GET /transactions/{id}", ctrl.ListTransactions)
	mux.HandleFunc("POST /payments/validate", ctrl.PayWithMethod("VALIDATE"))
	mux.HandleFunc("POST /payments/pix", ctrl.PayWithMethod("PIX"))
	mux.HandleFunc("POST /payments/qrcode", ctrl.PayWithMethod("QRCODE"))
	mux.HandleFunc("POST /payments/link", ctrl.PayWithMethod("LINK"))
	mux.HandleFunc("POST /payments/card", ctrl.PayWithMethod("CARD"))
	mux.HandleFunc("POST /payments/{id}/approve", ctrl.UpdatePaymentStatus(models.PaymentApproved))
	mux.HandleFunc("POST /payments/{id}/reject", ctrl.UpdatePaymentStatus(models.PaymentRejected))
	mux.HandleFunc("POST /payments/{id}/pending", ctrl.UpdatePaymentStatus(models.PaymentPending))
	mux.HandleFunc("POST /payments/{id}/refund", ctrl.UpdatePaymentStatus(models.PaymentRefunded))
	mux.HandleFunc("POST /webhooks/gatebox", ctrl.ReceiveGateboxWebhook)
	mux.HandleFunc("GET /simulation/settings", ctrl.GetSimulationSettings)
	mux.HandleFunc("PUT /simulation/settings", ctrl.PutSimulationSettings)

	return requestLogger(logger, mux)
}

func requestLogger(logger *log.Logger, next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()
		next.ServeHTTP(w, r)
		logger.Printf("%s %s duration=%s", r.Method, r.URL.Path, time.Since(start))
	})
}

