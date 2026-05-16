package user_bank_accountsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type User_bank_accountsRepositoryIF interface {
     GetUser_bank_accounts(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetUser_bank_accountsById(ctx context.Context, id int64) (*model.User_bank_accounts, error)
     GetUser_bank_accountsByBankAccountCode(ctx context.Context, bankaccountcode string) (*model.User_bank_accounts, error)
     InsertUser_bank_accounts(ctx context.Context, user_bank_accounts *model.User_bank_accounts) (int64, error)
     UpdateUser_bank_accounts(ctx context.Context, user_bank_accounts *model.User_bank_accounts, id int64) error
     DeleteUser_bank_accountsById(ctx context.Context, id int64) (bool, error)
}
 type User_bank_accountsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewUser_bank_accountsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *User_bank_accountsRepository{
    return &User_bank_accountsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("User_bank_accounts"),
     }
}
func (t User_bank_accountsRepository)  GetUser_bank_accounts(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_bank_accountsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_bank_accounts")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_bank_accounts.offset", offset)
	tracker.AddParam("repository.GetUser_bank_accounts.limit", limit)
	itemsPage 			= model.ItemsPage{}
	user_bank_accountss := []model.User_bank_accounts{}

	rows, err := t.PGRead.Query(ctx, SQL_USER_BANK_ACCOUNTS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "User_bank_accountsRepository.repository.GetUser_bank_accountss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var user_bank_accounts model.User_bank_accounts
		err := rows.Scan(
			&user_bank_accounts.ID,
			&user_bank_accounts.BankAccountCode,
			&user_bank_accounts.UserId,
			&user_bank_accounts.WalletId,
			&user_bank_accounts.IdBank,
			&user_bank_accounts.Agency,
			&user_bank_accounts.AccountNumber,
			&user_bank_accounts.IdAccountType,
			&user_bank_accounts.HolderName,
			&user_bank_accounts.HolderDocument,
			&user_bank_accounts.ConsentId,
			&user_bank_accounts.ConsentExpiresAt,
			&user_bank_accounts.IsVerified,
			&user_bank_accounts.IsActive,
			&user_bank_accounts.LastUsed,
			&user_bank_accounts.CreatedAt,
			&user_bank_accounts.UpdatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "User_bank_accountsRepository.repository.GetUser_bank_accountss.Scan: ", err.Error())
			return itemsPage, err
		}
		user_bank_accountss = append(user_bank_accountss, user_bank_accounts)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "User_bank_accountsRepository.repository.GetUser_bank_accountss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(user_bank_accountss) > 0 {
		qtyRecords = user_bank_accountss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = user_bank_accountss

	tracker.AddResult("repository.GetUser_bank_accounts.rows_returned", len(user_bank_accountss))
	tracker.AddResult("repository.GetUser_bank_accounts.total_count", len(user_bank_accountss))

	return itemsPage, nil
}
func (t User_bank_accountsRepository)  GetUser_bank_accountsById(ctx context.Context, id int64) (user_bank_accounts *model.User_bank_accounts, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_bank_accountsRepository -> GetUser_bank_accountsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_bank_accountsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_bank_accountsById.id", id)

	user_bank_accounts = new(model.User_bank_accounts)
	row := t.PGRead.QueryRow(ctx, SQL_GET_USER_BANK_ACCOUNTS_BY_ID, id)
		err = row.Scan(
			&user_bank_accounts.ID,
			&user_bank_accounts.BankAccountCode,
			&user_bank_accounts.UserId,
			&user_bank_accounts.WalletId,
			&user_bank_accounts.IdBank,
			&user_bank_accounts.Agency,
			&user_bank_accounts.AccountNumber,
			&user_bank_accounts.IdAccountType,
			&user_bank_accounts.HolderName,
			&user_bank_accounts.HolderDocument,
			&user_bank_accounts.ConsentId,
			&user_bank_accounts.ConsentExpiresAt,
			&user_bank_accounts.IsVerified,
			&user_bank_accounts.IsActive,
			&user_bank_accounts.LastUsed,
			&user_bank_accounts.CreatedAt,
			&user_bank_accounts.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"User_bank_accountsRepository.repository.GetUser_bank_accountsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetUser_bank_accountsById.found", true)
	return user_bank_accounts, nil
}
func (t User_bank_accountsRepository)  GetUser_bank_accountsByBankAccountCode(ctx context.Context, bankaccountcode string) (user_bank_accounts *model.User_bank_accounts, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_bank_accountsRepository -> GetUser_bank_accountsByBankAccountCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_bank_accountsByBankAccountCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_bank_accountsByBankAccountCode.bankaccountcode", bankaccountcode)

	user_bank_accounts = new(model.User_bank_accounts)
	row := t.PGRead.QueryRow(ctx, SQL_GET_USER_BANK_ACCOUNTS_BY_BANK_ACCOUNT_CODE, bankaccountcode)
		err = row.Scan(
			&user_bank_accounts.ID,
			&user_bank_accounts.BankAccountCode,
			&user_bank_accounts.UserId,
			&user_bank_accounts.WalletId,
			&user_bank_accounts.IdBank,
			&user_bank_accounts.Agency,
			&user_bank_accounts.AccountNumber,
			&user_bank_accounts.IdAccountType,
			&user_bank_accounts.HolderName,
			&user_bank_accounts.HolderDocument,
			&user_bank_accounts.ConsentId,
			&user_bank_accounts.ConsentExpiresAt,
			&user_bank_accounts.IsVerified,
			&user_bank_accounts.IsActive,
			&user_bank_accounts.LastUsed,
			&user_bank_accounts.CreatedAt,
			&user_bank_accounts.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"User_bank_accountsRepository.repository.GetUser_bank_accountsBybankaccountcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return user_bank_accounts, nil
}
func (t User_bank_accountsRepository)  DeleteUser_bank_accountsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_bank_accountsRepository -> DeleteUser_bank_accountsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteUser_bank_accountsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_USER_BANK_ACCOUNTS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"User_bank_accountsRepository.repository.DeleteUser_bank_accountsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteUser_bank_accountsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteUser_bank_accountsById.deleted", result)
	return true, err
}
func (t User_bank_accountsRepository)  InsertUser_bank_accounts(ctx context.Context,user_bank_accounts *model.User_bank_accounts) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_bank_accountsRepository -> InsertUser_bank_accounts", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertUser_bank_accounts")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertUser_bank_accounts.bankaccountcode", user_bank_accounts.BankAccountCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_USER_BANK_ACCOUNTS_INSERT,
			user_bank_accounts.BankAccountCode,
			user_bank_accounts.UserId,
			user_bank_accounts.WalletId,
			user_bank_accounts.IdBank,
			user_bank_accounts.Agency,
			user_bank_accounts.AccountNumber,
			user_bank_accounts.IdAccountType,
			user_bank_accounts.HolderName,
			user_bank_accounts.HolderDocument,
			user_bank_accounts.ConsentId,
			user_bank_accounts.ConsentExpiresAt,
			user_bank_accounts.IsVerified,
			user_bank_accounts.IsActive,
			user_bank_accounts.LastUsed,
			user_bank_accounts.CreatedAt,
			user_bank_accounts.UpdatedAt,
	).Scan(&user_bank_accounts.ID)

	if err != nil {
		t.log.Error(ctx, "User_bank_accountsRepository.repository.InsertUser_bank_accounts.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertUser_bank_accounts.inserted_id", user_bank_accounts.ID)
   return user_bank_accounts.ID, nil

}
func (t User_bank_accountsRepository)  UpdateUser_bank_accounts(ctx context.Context,user_bank_accounts *model.User_bank_accounts, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_bank_accountsRepository -> UpdateUser_bank_accounts", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateUser_bank_accounts")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateUser_bank_accounts.id", id)
	tracker.AddParam("repository.UpdateUser_bank_accounts.bankaccountcode", user_bank_accounts.BankAccountCode)

	user_bank_accounts.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_USER_BANK_ACCOUNTS_UPDATE, 
			user_bank_accounts.BankAccountCode,
			user_bank_accounts.UserId,
			user_bank_accounts.WalletId,
			user_bank_accounts.IdBank,
			user_bank_accounts.Agency,
			user_bank_accounts.AccountNumber,
			user_bank_accounts.IdAccountType,
			user_bank_accounts.HolderName,
			user_bank_accounts.HolderDocument,
			user_bank_accounts.ConsentId,
			user_bank_accounts.ConsentExpiresAt,
			user_bank_accounts.IsVerified,
			user_bank_accounts.IsActive,
			user_bank_accounts.LastUsed,
			user_bank_accounts.CreatedAt,
			user_bank_accounts.UpdatedAt,
			user_bank_accounts.ID,
   )
	if err != nil {
		t.log.Error(ctx, "User_bank_accountsRepository.repository.UpdateUser_bank_accounts.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateUser_bank_accounts.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateUser_bank_accounts.rows_affected", rowsAffected)
	return nil
}

