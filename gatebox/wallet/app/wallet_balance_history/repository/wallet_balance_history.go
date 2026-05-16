package wallet_balance_historyRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Wallet_balance_historyRepositoryIF interface {
     GetWallet_balance_history(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetWallet_balance_historyById(ctx context.Context, id int64) (*model.Wallet_balance_history, error)
     GetWallet_balance_historyByBalanceHistoryCode(ctx context.Context, balancehistorycode string) (*model.Wallet_balance_history, error)
     InsertWallet_balance_history(ctx context.Context, wallet_balance_history *model.Wallet_balance_history) (int64, error)
     UpdateWallet_balance_history(ctx context.Context, wallet_balance_history *model.Wallet_balance_history, id int64) error
     DeleteWallet_balance_historyById(ctx context.Context, id int64) (bool, error)
}
 type Wallet_balance_historyRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewWallet_balance_historyRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Wallet_balance_historyRepository{
    return &Wallet_balance_historyRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Wallet_balance_history"),
     }
}
func (t Wallet_balance_historyRepository)  GetWallet_balance_history(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_balance_historyRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetWallet_balance_history")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetWallet_balance_history.offset", offset)
	tracker.AddParam("repository.GetWallet_balance_history.limit", limit)
	itemsPage 			= model.ItemsPage{}
	wallet_balance_historys := []model.Wallet_balance_history{}

	rows, err := t.PGRead.Query(ctx, SQL_WALLET_BALANCE_HISTORY_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Wallet_balance_historyRepository.repository.GetWallet_balance_historys.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var wallet_balance_history model.Wallet_balance_history
		err := rows.Scan(
			&wallet_balance_history.ID,
			&wallet_balance_history.BalanceHistoryCode,
			&wallet_balance_history.WalletId,
			&wallet_balance_history.PreviousBalance,
			&wallet_balance_history.NewBalance,
			&wallet_balance_history.ChangeAmount,
			&wallet_balance_history.IdChangeType,
			&wallet_balance_history.ReferenceId,
			&wallet_balance_history.Description,
			&wallet_balance_history.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Wallet_balance_historyRepository.repository.GetWallet_balance_historys.Scan: ", err.Error())
			return itemsPage, err
		}
		wallet_balance_historys = append(wallet_balance_historys, wallet_balance_history)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Wallet_balance_historyRepository.repository.GetWallet_balance_historys.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(wallet_balance_historys) > 0 {
		qtyRecords = wallet_balance_historys[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = wallet_balance_historys

	tracker.AddResult("repository.GetWallet_balance_history.rows_returned", len(wallet_balance_historys))
	tracker.AddResult("repository.GetWallet_balance_history.total_count", len(wallet_balance_historys))

	return itemsPage, nil
}
func (t Wallet_balance_historyRepository)  GetWallet_balance_historyById(ctx context.Context, id int64) (wallet_balance_history *model.Wallet_balance_history, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_balance_historyRepository -> GetWallet_balance_historyById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetWallet_balance_historyById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetWallet_balance_historyById.id", id)

	wallet_balance_history = new(model.Wallet_balance_history)
	row := t.PGRead.QueryRow(ctx, SQL_GET_WALLET_BALANCE_HISTORY_BY_ID, id)
		err = row.Scan(
			&wallet_balance_history.ID,
			&wallet_balance_history.BalanceHistoryCode,
			&wallet_balance_history.WalletId,
			&wallet_balance_history.PreviousBalance,
			&wallet_balance_history.NewBalance,
			&wallet_balance_history.ChangeAmount,
			&wallet_balance_history.IdChangeType,
			&wallet_balance_history.ReferenceId,
			&wallet_balance_history.Description,
			&wallet_balance_history.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Wallet_balance_historyRepository.repository.GetWallet_balance_historyById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetWallet_balance_historyById.found", true)
	return wallet_balance_history, nil
}
func (t Wallet_balance_historyRepository)  GetWallet_balance_historyByBalanceHistoryCode(ctx context.Context, balancehistorycode string) (wallet_balance_history *model.Wallet_balance_history, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_balance_historyRepository -> GetWallet_balance_historyByBalanceHistoryCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetWallet_balance_historyByBalanceHistoryCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetWallet_balance_historyByBalanceHistoryCode.balancehistorycode", balancehistorycode)

	wallet_balance_history = new(model.Wallet_balance_history)
	row := t.PGRead.QueryRow(ctx, SQL_GET_WALLET_BALANCE_HISTORY_BY_BALANCE_HISTORY_CODE, balancehistorycode)
		err = row.Scan(
			&wallet_balance_history.ID,
			&wallet_balance_history.BalanceHistoryCode,
			&wallet_balance_history.WalletId,
			&wallet_balance_history.PreviousBalance,
			&wallet_balance_history.NewBalance,
			&wallet_balance_history.ChangeAmount,
			&wallet_balance_history.IdChangeType,
			&wallet_balance_history.ReferenceId,
			&wallet_balance_history.Description,
			&wallet_balance_history.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Wallet_balance_historyRepository.repository.GetWallet_balance_historyBybalancehistorycode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return wallet_balance_history, nil
}
func (t Wallet_balance_historyRepository)  DeleteWallet_balance_historyById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_balance_historyRepository -> DeleteWallet_balance_historyById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteWallet_balance_historyById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_WALLET_BALANCE_HISTORY_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Wallet_balance_historyRepository.repository.DeleteWallet_balance_historyById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteWallet_balance_historyById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteWallet_balance_historyById.deleted", result)
	return true, err
}
func (t Wallet_balance_historyRepository)  InsertWallet_balance_history(ctx context.Context,wallet_balance_history *model.Wallet_balance_history) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_balance_historyRepository -> InsertWallet_balance_history", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertWallet_balance_history")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertWallet_balance_history.balancehistorycode", wallet_balance_history.BalanceHistoryCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_WALLET_BALANCE_HISTORY_INSERT,
			wallet_balance_history.BalanceHistoryCode,
			wallet_balance_history.WalletId,
			wallet_balance_history.PreviousBalance,
			wallet_balance_history.NewBalance,
			wallet_balance_history.ChangeAmount,
			wallet_balance_history.IdChangeType,
			wallet_balance_history.ReferenceId,
			wallet_balance_history.Description,
			wallet_balance_history.CreatedAt,
	).Scan(&wallet_balance_history.ID)

	if err != nil {
		t.log.Error(ctx, "Wallet_balance_historyRepository.repository.InsertWallet_balance_history.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertWallet_balance_history.inserted_id", wallet_balance_history.ID)
   return wallet_balance_history.ID, nil

}
func (t Wallet_balance_historyRepository)  UpdateWallet_balance_history(ctx context.Context,wallet_balance_history *model.Wallet_balance_history, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_balance_historyRepository -> UpdateWallet_balance_history", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateWallet_balance_history")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateWallet_balance_history.id", id)
	tracker.AddParam("repository.UpdateWallet_balance_history.balancehistorycode", wallet_balance_history.BalanceHistoryCode)

	wallet_balance_history.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_WALLET_BALANCE_HISTORY_UPDATE, 
			wallet_balance_history.BalanceHistoryCode,
			wallet_balance_history.WalletId,
			wallet_balance_history.PreviousBalance,
			wallet_balance_history.NewBalance,
			wallet_balance_history.ChangeAmount,
			wallet_balance_history.IdChangeType,
			wallet_balance_history.ReferenceId,
			wallet_balance_history.Description,
			wallet_balance_history.CreatedAt,
			wallet_balance_history.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Wallet_balance_historyRepository.repository.UpdateWallet_balance_history.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateWallet_balance_history.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateWallet_balance_history.rows_affected", rowsAffected)
	return nil
}

