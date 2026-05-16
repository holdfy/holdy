package wallet_statusRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Wallet_statusRepositoryIF interface {
     GetWallet_status(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetWallet_statusById(ctx context.Context, id int64) (*model.Wallet_status, error)
     GetWallet_statusByStatusCode(ctx context.Context, statuscode string) (*model.Wallet_status, error)
     InsertWallet_status(ctx context.Context, wallet_status *model.Wallet_status) (int64, error)
     UpdateWallet_status(ctx context.Context, wallet_status *model.Wallet_status, id int64) error
     DeleteWallet_statusById(ctx context.Context, id int64) (bool, error)
}
 type Wallet_statusRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewWallet_statusRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Wallet_statusRepository{
    return &Wallet_statusRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Wallet_status"),
     }
}
func (t Wallet_statusRepository)  GetWallet_status(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_statusRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetWallet_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetWallet_status.offset", offset)
	tracker.AddParam("repository.GetWallet_status.limit", limit)
	itemsPage 			= model.ItemsPage{}
	wallet_statuss := []model.Wallet_status{}

	rows, err := t.PGRead.Query(ctx, SQL_WALLET_STATUS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Wallet_statusRepository.repository.GetWallet_statuss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var wallet_status model.Wallet_status
		err := rows.Scan(
			&wallet_status.ID,
			&wallet_status.StatusCode,
			&wallet_status.Name,
			&wallet_status.Description,
			&wallet_status.AllowsTransactions,
			&wallet_status.AllowsDeposits,
			&wallet_status.AllowsWithdrawals,
			&wallet_status.IsActive,
			&wallet_status.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Wallet_statusRepository.repository.GetWallet_statuss.Scan: ", err.Error())
			return itemsPage, err
		}
		wallet_statuss = append(wallet_statuss, wallet_status)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Wallet_statusRepository.repository.GetWallet_statuss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(wallet_statuss) > 0 {
		qtyRecords = wallet_statuss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = wallet_statuss

	tracker.AddResult("repository.GetWallet_status.rows_returned", len(wallet_statuss))
	tracker.AddResult("repository.GetWallet_status.total_count", len(wallet_statuss))

	return itemsPage, nil
}
func (t Wallet_statusRepository)  GetWallet_statusById(ctx context.Context, id int64) (wallet_status *model.Wallet_status, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_statusRepository -> GetWallet_statusById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetWallet_statusById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetWallet_statusById.id", id)

	wallet_status = new(model.Wallet_status)
	row := t.PGRead.QueryRow(ctx, SQL_GET_WALLET_STATUS_BY_ID, id)
		err = row.Scan(
			&wallet_status.ID,
			&wallet_status.StatusCode,
			&wallet_status.Name,
			&wallet_status.Description,
			&wallet_status.AllowsTransactions,
			&wallet_status.AllowsDeposits,
			&wallet_status.AllowsWithdrawals,
			&wallet_status.IsActive,
			&wallet_status.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Wallet_statusRepository.repository.GetWallet_statusById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetWallet_statusById.found", true)
	return wallet_status, nil
}
func (t Wallet_statusRepository)  GetWallet_statusByStatusCode(ctx context.Context, statuscode string) (wallet_status *model.Wallet_status, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_statusRepository -> GetWallet_statusByStatusCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetWallet_statusByStatusCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetWallet_statusByStatusCode.statuscode", statuscode)

	wallet_status = new(model.Wallet_status)
	row := t.PGRead.QueryRow(ctx, SQL_GET_WALLET_STATUS_BY_STATUS_CODE, statuscode)
		err = row.Scan(
			&wallet_status.ID,
			&wallet_status.StatusCode,
			&wallet_status.Name,
			&wallet_status.Description,
			&wallet_status.AllowsTransactions,
			&wallet_status.AllowsDeposits,
			&wallet_status.AllowsWithdrawals,
			&wallet_status.IsActive,
			&wallet_status.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Wallet_statusRepository.repository.GetWallet_statusBystatuscode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return wallet_status, nil
}
func (t Wallet_statusRepository)  DeleteWallet_statusById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_statusRepository -> DeleteWallet_statusById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteWallet_statusById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_WALLET_STATUS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Wallet_statusRepository.repository.DeleteWallet_statusById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteWallet_statusById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteWallet_statusById.deleted", result)
	return true, err
}
func (t Wallet_statusRepository)  InsertWallet_status(ctx context.Context,wallet_status *model.Wallet_status) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_statusRepository -> InsertWallet_status", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertWallet_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertWallet_status.statuscode", wallet_status.StatusCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_WALLET_STATUS_INSERT,
			wallet_status.StatusCode,
			wallet_status.Name,
			wallet_status.Description,
			wallet_status.AllowsTransactions,
			wallet_status.AllowsDeposits,
			wallet_status.AllowsWithdrawals,
			wallet_status.IsActive,
			wallet_status.CreatedAt,
	).Scan(&wallet_status.ID)

	if err != nil {
		t.log.Error(ctx, "Wallet_statusRepository.repository.InsertWallet_status.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertWallet_status.inserted_id", wallet_status.ID)
   return wallet_status.ID, nil

}
func (t Wallet_statusRepository)  UpdateWallet_status(ctx context.Context,wallet_status *model.Wallet_status, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_statusRepository -> UpdateWallet_status", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateWallet_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateWallet_status.id", id)
	tracker.AddParam("repository.UpdateWallet_status.statuscode", wallet_status.StatusCode)

	wallet_status.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_WALLET_STATUS_UPDATE, 
			wallet_status.StatusCode,
			wallet_status.Name,
			wallet_status.Description,
			wallet_status.AllowsTransactions,
			wallet_status.AllowsDeposits,
			wallet_status.AllowsWithdrawals,
			wallet_status.IsActive,
			wallet_status.CreatedAt,
			wallet_status.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Wallet_statusRepository.repository.UpdateWallet_status.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateWallet_status.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateWallet_status.rows_affected", rowsAffected)
	return nil
}

