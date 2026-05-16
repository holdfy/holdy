package config

import "os"

type Config struct {
	HTTPAddr           string
	DatabaseURL        string
	JWTSecret          string
	GateboxBaseURL     string
	GateboxAPIKey      string
	Environment        string
	DefaultWebhookURL  string
	DefaultSimBehavior string
}

func Load() Config {
	return Config{
		HTTPAddr:           env("BANCO_HTTP_ADDR", ":8091"),
		DatabaseURL:        env("BANCO_DATABASE_URL", "postgres://postgres:postgres@127.0.0.1:15432/airbnb_ia?sslmode=disable"),
		JWTSecret:          env("BANCO_JWT_SECRET", "change-me"),
		// Alinhar com money/runapp.sh: Gatebox Rust escuta GB_API_PORT (defeito 8081), não 8080.
		GateboxBaseURL:    env("GATEBOX_BASE_URL", "http://127.0.0.1:8081"),
		GateboxAPIKey:     env("GATEBOX_API_KEY", "sandbox-key"),
		Environment:       env("BANCO_ENVIRONMENT", "sandbox"),
		DefaultWebhookURL: env("BANCO_WEBHOOK_URL", "http://127.0.0.1:8081/internal/bank/webhooks"),
		DefaultSimBehavior: env("BANCO_DEFAULT_SIM_BEHAVIOR", "manual"),
	}
}

func env(k, fallback string) string {
	if v := os.Getenv(k); v != "" {
		return v
	}
	return fallback
}

