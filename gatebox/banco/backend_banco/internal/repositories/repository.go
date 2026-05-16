package repositories

import (
	"context"
	"database/sql"
	"time"

	"banco_saczuck_backend/internal/models"
)

type Repository struct {
	db *sql.DB
}

func New(db *sql.DB) *Repository {
	return &Repository{db: db}
}

func (r *Repository) CreateAccount(ctx context.Context, a models.Account, passwordHash string) error {
	now := time.Now().UTC().Format(time.RFC3339)
	tx, err := r.db.BeginTx(ctx, nil)
	if err != nil {
		return err
	}
	defer tx.Rollback()
	_, err = tx.ExecContext(ctx, `INSERT INTO banco_users (id, full_name, person_type, document, email, password_hash, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)`,
		a.UserID, a.FullName, a.PersonType, a.Document, a.Email, passwordHash, now)
	if err != nil {
		return err
	}
	_, err = tx.ExecContext(ctx, `INSERT INTO banco_bank_accounts (id, user_id, agency, account_number, pix_key, status, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)`,
		a.ID, a.UserID, a.Agency, a.AccountNumber, a.PixKey, a.Status, now)
	if err != nil {
		return err
	}
	_, err = tx.ExecContext(ctx, `INSERT INTO banco_balances (account_id, available_cents, blocked_cents, updated_at) VALUES ($1, 0, 0, $2)`, a.ID, now)
	if err != nil {
		return err
	}
	_, err = tx.ExecContext(ctx, `INSERT INTO banco_simulation_settings (account_id, updated_at) VALUES ($1, $2)`, a.ID, now)
	if err != nil {
		return err
	}
	return tx.Commit()
}

func (r *Repository) GetAccountByUserID(ctx context.Context, userID string) (models.Account, error) {
	var a models.Account
	err := r.db.QueryRowContext(ctx, `
SELECT ba.id, ba.user_id, u.full_name, u.person_type, u.document, u.email, ba.agency, ba.account_number, ba.pix_key, ba.status
FROM banco_bank_accounts ba
JOIN banco_users u ON u.id = ba.user_id
WHERE ba.user_id = $1`, userID).Scan(&a.ID, &a.UserID, &a.FullName, &a.PersonType, &a.Document, &a.Email, &a.Agency, &a.AccountNumber, &a.PixKey, &a.Status)
	return a, err
}

func (r *Repository) GetAccountByBankData(ctx context.Context, agency, accountNumber, personType, document string) (models.Account, error) {
	var a models.Account
	err := r.db.QueryRowContext(ctx, `
SELECT ba.id, ba.user_id, u.full_name, u.person_type, u.document, u.email, ba.agency, ba.account_number, ba.pix_key, ba.status
FROM banco_bank_accounts ba
JOIN banco_users u ON u.id = ba.user_id
WHERE ba.agency = $1 AND ba.account_number = $2 AND u.person_type = $3 AND u.document = $4`,
		agency, accountNumber, personType, document).
		Scan(&a.ID, &a.UserID, &a.FullName, &a.PersonType, &a.Document, &a.Email, &a.Agency, &a.AccountNumber, &a.PixKey, &a.Status)
	return a, err
}

func (r *Repository) GetBalance(ctx context.Context, accountID string) (models.Balance, error) {
	var b models.Balance
	err := r.db.QueryRowContext(ctx, `SELECT account_id, available_cents, blocked_cents FROM banco_balances WHERE account_id = $1`, accountID).
		Scan(&b.AccountID, &b.AvailableCents, &b.BlockedCents)
	return b, err
}

func (r *Repository) AddTopup(ctx context.Context, accountID string, amount int64, details string) error {
	now := time.Now().UTC().Format(time.RFC3339)
	tx, err := r.db.BeginTx(ctx, nil)
	if err != nil {
		return err
	}
	defer tx.Rollback()
	if _, err := tx.ExecContext(ctx, `UPDATE banco_balances SET available_cents = available_cents + $1, updated_at = $2 WHERE account_id = $3`, amount, now, accountID); err != nil {
		return err
	}
	if _, err := tx.ExecContext(ctx, `INSERT INTO banco_transactions (id, account_id, type, amount_cents, status, details, created_at) VALUES (md5(random()::text || clock_timestamp()::text), $1, 'TOPUP', $2, 'APPROVED', $3, $4)`,
		accountID, amount, details, now); err != nil {
		return err
	}
	return tx.Commit()
}

func (r *Repository) GetCredentialsByEmail(ctx context.Context, email string) (userID string, passwordHash string, err error) {
	err = r.db.QueryRowContext(ctx, `SELECT id, password_hash FROM banco_users WHERE email = $1`, email).Scan(&userID, &passwordHash)
	return
}

