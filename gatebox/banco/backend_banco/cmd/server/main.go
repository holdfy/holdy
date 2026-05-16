package main

import (
	"context"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"banco_saczuck_backend/internal/config"
	"banco_saczuck_backend/internal/database"
	"banco_saczuck_backend/internal/httpapi"
)

func main() {
	cfg := config.Load()
	logger := log.New(os.Stdout, "[banco-saczuck] ", log.LstdFlags|log.LUTC|log.Lshortfile)

	db, err := database.OpenAndMigrate(cfg.DatabaseURL)
	if err != nil {
		logger.Fatalf("database error: %v", err)
	}
	defer db.Close()

	handler := httpapi.NewRouter(cfg, db, logger)
	srv := &http.Server{
		Addr:              cfg.HTTPAddr,
		Handler:           handler,
		ReadHeaderTimeout: 8 * time.Second,
	}

	go func() {
		logger.Printf("server listening on %s", cfg.HTTPAddr)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			logger.Fatalf("server error: %v", err)
		}
	}()

	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt, syscall.SIGTERM)
	<-stop

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	_ = srv.Shutdown(ctx)
}

