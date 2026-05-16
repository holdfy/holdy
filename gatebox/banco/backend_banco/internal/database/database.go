package database

import (
	"database/sql"
	"embed"
	"fmt"
	"sort"
	"strings"

	_ "github.com/jackc/pgx/v5/stdlib"
)

//go:embed migrations/*.sql
var migrationFiles embed.FS

func OpenAndMigrate(databaseURL string) (*sql.DB, error) {
	db, err := sql.Open("pgx", databaseURL)
	if err != nil {
		return nil, err
	}
	if err := db.Ping(); err != nil {
		return nil, err
	}
	if err := migrate(db); err != nil {
		return nil, err
	}
	return db, nil
}

func migrate(db *sql.DB) error {
	if _, err := db.Exec(`CREATE TABLE IF NOT EXISTS banco_schema_migrations (name TEXT PRIMARY KEY);`); err != nil {
		return err
	}

	entries, err := migrationFiles.ReadDir("migrations")
	if err != nil {
		return err
	}
	names := make([]string, 0, len(entries))
	for _, e := range entries {
		if !e.IsDir() && strings.HasSuffix(e.Name(), ".sql") {
			names = append(names, e.Name())
		}
	}
	sort.Strings(names)

	for _, name := range names {
		var exists string
		err := db.QueryRow(`SELECT name FROM banco_schema_migrations WHERE name = $1`, name).Scan(&exists)
		if err == nil {
			continue
		}
		if err != sql.ErrNoRows {
			return err
		}
		sqlBytes, err := migrationFiles.ReadFile("migrations/" + name)
		if err != nil {
			return err
		}
		if _, err := db.Exec(string(sqlBytes)); err != nil {
			return fmt.Errorf("migration %s failed: %w", name, err)
		}
		if _, err := db.Exec(`INSERT INTO banco_schema_migrations (name) VALUES ($1)`, name); err != nil {
			return err
		}
	}
	return nil
}

