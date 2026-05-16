package transaction_statusRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Transaction_statusRepositoryIF interface {
     GetTransaction_status(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetTransaction_statusById(ctx context.Context, id int64) (*model.Transaction_status, error)
     GetTransaction_statusByStatusCode(ctx context.Context, statuscode string) (*model.Transaction_status, error)
     InsertTransaction_status(ctx context.Context, transaction_status *model.Transaction_status) (int64, error)
     UpdateTransaction_status(ctx context.Context, transaction_status *model.Transaction_status, id int64) error
     DeleteTransaction_statusById(ctx context.Context, id int64) (bool, error)
}
 type Transaction_statusRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewTransaction_statusRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Transaction_statusRepository{
    return &Transaction_statusRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Transaction_status"),
     }
}
func (t Transaction_statusRepository)  GetTransaction_status(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_statusRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetTransaction_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetTransaction_status.offset", offset)
	tracker.AddParam("repository.GetTransaction_status.limit", limit)
	itemsPage 			= model.ItemsPage{}
	transaction_statuss := []model.Transaction_status{}

	rows, err := t.PGRead.Query(ctx, SQL_TRANSACTION_STATUS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Transaction_statusRepository.repository.GetTransaction_statuss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var transaction_status model.Transaction_status
		err := rows.Scan(
			&transaction_status.ID,
			&transaction_status.StatusCode,
			&transaction_status.Name,
			&transaction_status.Description,
			&transaction_status.IsFinal,
			&transaction_status.IsSuccess,
			&transaction_status.AllowsRefund,
			&transaction_status.IsActive,
			&transaction_status.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Transaction_statusRepository.repository.GetTransaction_statuss.Scan: ", err.Error())
			return itemsPage, err
		}
		transaction_statuss = append(transaction_statuss, transaction_status)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Transaction_statusRepository.repository.GetTransaction_statuss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(transaction_statuss) > 0 {
		qtyRecords = transaction_statuss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = transaction_statuss

	tracker.AddResult("repository.GetTransaction_status.rows_returned", len(transaction_statuss))
	tracker.AddResult("repository.GetTransaction_status.total_count", len(transaction_statuss))

	return itemsPage, nil
}
func (t Transaction_statusRepository)  GetTransaction_statusById(ctx context.Context, id int64) (transaction_status *model.Transaction_status, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_statusRepository -> GetTransaction_statusById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetTransaction_statusById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetTransaction_statusById.id", id)

	transaction_status = new(model.Transaction_status)
	row := t.PGRead.QueryRow(ctx, SQL_GET_TRANSACTION_STATUS_BY_ID, id)
		err = row.Scan(
			&transaction_status.ID,
			&transaction_status.StatusCode,
			&transaction_status.Name,
			&transaction_status.Description,
			&transaction_status.IsFinal,
			&transaction_status.IsSuccess,
			&transaction_status.AllowsRefund,
			&transaction_status.IsActive,
			&transaction_status.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Transaction_statusRepository.repository.GetTransaction_statusById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetTransaction_statusById.found", true)
	return transaction_status, nil
}
func (t Transaction_statusRepository)  GetTransaction_statusByStatusCode(ctx context.Context, statuscode string) (transaction_status *model.Transaction_status, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_statusRepository -> GetTransaction_statusByStatusCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetTransaction_statusByStatusCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetTransaction_statusByStatusCode.statuscode", statuscode)

	transaction_status = new(model.Transaction_status)
	row := t.PGRead.QueryRow(ctx, SQL_GET_TRANSACTION_STATUS_BY_STATUS_CODE, statuscode)
		err = row.Scan(
			&transaction_status.ID,
			&transaction_status.StatusCode,
			&transaction_status.Name,
			&transaction_status.Description,
			&transaction_status.IsFinal,
			&transaction_status.IsSuccess,
			&transaction_status.AllowsRefund,
			&transaction_status.IsActive,
			&transaction_status.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Transaction_statusRepository.repository.GetTransaction_statusBystatuscode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return transaction_status, nil
}
func (t Transaction_statusRepository)  DeleteTransaction_statusById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_statusRepository -> DeleteTransaction_statusById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteTransaction_statusById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_TRANSACTION_STATUS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Transaction_statusRepository.repository.DeleteTransaction_statusById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteTransaction_statusById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteTransaction_statusById.deleted", result)
	return true, err
}
func (t Transaction_statusRepository)  InsertTransaction_status(ctx context.Context,transaction_status *model.Transaction_status) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_statusRepository -> InsertTransaction_status", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertTransaction_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertTransaction_status.statuscode", transaction_status.StatusCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_TRANSACTION_STATUS_INSERT,
			transaction_status.StatusCode,
			transaction_status.Name,
			transaction_status.Description,
			transaction_status.IsFinal,
			transaction_status.IsSuccess,
			transaction_status.AllowsRefund,
			transaction_status.IsActive,
			transaction_status.CreatedAt,
	).Scan(&transaction_status.ID)

	if err != nil {
		t.log.Error(ctx, "Transaction_statusRepository.repository.InsertTransaction_status.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertTransaction_status.inserted_id", transaction_status.ID)
   return transaction_status.ID, nil

}
func (t Transaction_statusRepository)  UpdateTransaction_status(ctx context.Context,transaction_status *model.Transaction_status, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_statusRepository -> UpdateTransaction_status", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateTransaction_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateTransaction_status.id", id)
	tracker.AddParam("repository.UpdateTransaction_status.statuscode", transaction_status.StatusCode)

	transaction_status.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_TRANSACTION_STATUS_UPDATE, 
			transaction_status.StatusCode,
			transaction_status.Name,
			transaction_status.Description,
			transaction_status.IsFinal,
			transaction_status.IsSuccess,
			transaction_status.AllowsRefund,
			transaction_status.IsActive,
			transaction_status.CreatedAt,
			transaction_status.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Transaction_statusRepository.repository.UpdateTransaction_status.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateTransaction_status.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateTransaction_status.rows_affected", rowsAffected)
	return nil
}

