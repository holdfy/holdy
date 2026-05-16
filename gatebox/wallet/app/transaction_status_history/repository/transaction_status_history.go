package transaction_status_historyRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Transaction_status_historyRepositoryIF interface {
     GetTransaction_status_history(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetTransaction_status_historyById(ctx context.Context, id int64) (*model.Transaction_status_history, error)
     GetTransaction_status_historyByStatusHistoryCode(ctx context.Context, statushistorycode string) (*model.Transaction_status_history, error)
     InsertTransaction_status_history(ctx context.Context, transaction_status_history *model.Transaction_status_history) (int64, error)
     UpdateTransaction_status_history(ctx context.Context, transaction_status_history *model.Transaction_status_history, id int64) error
     DeleteTransaction_status_historyById(ctx context.Context, id int64) (bool, error)
}
 type Transaction_status_historyRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewTransaction_status_historyRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Transaction_status_historyRepository{
    return &Transaction_status_historyRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Transaction_status_history"),
     }
}
func (t Transaction_status_historyRepository)  GetTransaction_status_history(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_status_historyRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetTransaction_status_history")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetTransaction_status_history.offset", offset)
	tracker.AddParam("repository.GetTransaction_status_history.limit", limit)
	itemsPage 			= model.ItemsPage{}
	transaction_status_historys := []model.Transaction_status_history{}

	rows, err := t.PGRead.Query(ctx, SQL_TRANSACTION_STATUS_HISTORY_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Transaction_status_historyRepository.repository.GetTransaction_status_historys.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var transaction_status_history model.Transaction_status_history
		err := rows.Scan(
			&transaction_status_history.ID,
			&transaction_status_history.StatusHistoryCode,
			&transaction_status_history.TransactionId,
			&transaction_status_history.IdPreviousStatus,
			&transaction_status_history.IdNewStatus,
			&transaction_status_history.Reason,
			&transaction_status_history.IdChangedBy,
			&transaction_status_history.GatewayResponse,
			&transaction_status_history.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Transaction_status_historyRepository.repository.GetTransaction_status_historys.Scan: ", err.Error())
			return itemsPage, err
		}
		transaction_status_historys = append(transaction_status_historys, transaction_status_history)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Transaction_status_historyRepository.repository.GetTransaction_status_historys.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(transaction_status_historys) > 0 {
		qtyRecords = transaction_status_historys[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = transaction_status_historys

	tracker.AddResult("repository.GetTransaction_status_history.rows_returned", len(transaction_status_historys))
	tracker.AddResult("repository.GetTransaction_status_history.total_count", len(transaction_status_historys))

	return itemsPage, nil
}
func (t Transaction_status_historyRepository)  GetTransaction_status_historyById(ctx context.Context, id int64) (transaction_status_history *model.Transaction_status_history, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_status_historyRepository -> GetTransaction_status_historyById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetTransaction_status_historyById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetTransaction_status_historyById.id", id)

	transaction_status_history = new(model.Transaction_status_history)
	row := t.PGRead.QueryRow(ctx, SQL_GET_TRANSACTION_STATUS_HISTORY_BY_ID, id)
		err = row.Scan(
			&transaction_status_history.ID,
			&transaction_status_history.StatusHistoryCode,
			&transaction_status_history.TransactionId,
			&transaction_status_history.IdPreviousStatus,
			&transaction_status_history.IdNewStatus,
			&transaction_status_history.Reason,
			&transaction_status_history.IdChangedBy,
			&transaction_status_history.GatewayResponse,
			&transaction_status_history.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Transaction_status_historyRepository.repository.GetTransaction_status_historyById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetTransaction_status_historyById.found", true)
	return transaction_status_history, nil
}
func (t Transaction_status_historyRepository)  GetTransaction_status_historyByStatusHistoryCode(ctx context.Context, statushistorycode string) (transaction_status_history *model.Transaction_status_history, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_status_historyRepository -> GetTransaction_status_historyByStatusHistoryCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetTransaction_status_historyByStatusHistoryCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetTransaction_status_historyByStatusHistoryCode.statushistorycode", statushistorycode)

	transaction_status_history = new(model.Transaction_status_history)
	row := t.PGRead.QueryRow(ctx, SQL_GET_TRANSACTION_STATUS_HISTORY_BY_STATUS_HISTORY_CODE, statushistorycode)
		err = row.Scan(
			&transaction_status_history.ID,
			&transaction_status_history.StatusHistoryCode,
			&transaction_status_history.TransactionId,
			&transaction_status_history.IdPreviousStatus,
			&transaction_status_history.IdNewStatus,
			&transaction_status_history.Reason,
			&transaction_status_history.IdChangedBy,
			&transaction_status_history.GatewayResponse,
			&transaction_status_history.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Transaction_status_historyRepository.repository.GetTransaction_status_historyBystatushistorycode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return transaction_status_history, nil
}
func (t Transaction_status_historyRepository)  DeleteTransaction_status_historyById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_status_historyRepository -> DeleteTransaction_status_historyById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteTransaction_status_historyById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_TRANSACTION_STATUS_HISTORY_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Transaction_status_historyRepository.repository.DeleteTransaction_status_historyById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteTransaction_status_historyById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteTransaction_status_historyById.deleted", result)
	return true, err
}
func (t Transaction_status_historyRepository)  InsertTransaction_status_history(ctx context.Context,transaction_status_history *model.Transaction_status_history) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_status_historyRepository -> InsertTransaction_status_history", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertTransaction_status_history")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertTransaction_status_history.statushistorycode", transaction_status_history.StatusHistoryCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_TRANSACTION_STATUS_HISTORY_INSERT,
			transaction_status_history.StatusHistoryCode,
			transaction_status_history.TransactionId,
			transaction_status_history.IdPreviousStatus,
			transaction_status_history.IdNewStatus,
			transaction_status_history.Reason,
			transaction_status_history.IdChangedBy,
			transaction_status_history.GatewayResponse,
			transaction_status_history.CreatedAt,
	).Scan(&transaction_status_history.ID)

	if err != nil {
		t.log.Error(ctx, "Transaction_status_historyRepository.repository.InsertTransaction_status_history.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertTransaction_status_history.inserted_id", transaction_status_history.ID)
   return transaction_status_history.ID, nil

}
func (t Transaction_status_historyRepository)  UpdateTransaction_status_history(ctx context.Context,transaction_status_history *model.Transaction_status_history, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_status_historyRepository -> UpdateTransaction_status_history", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateTransaction_status_history")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateTransaction_status_history.id", id)
	tracker.AddParam("repository.UpdateTransaction_status_history.statushistorycode", transaction_status_history.StatusHistoryCode)

	transaction_status_history.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_TRANSACTION_STATUS_HISTORY_UPDATE, 
			transaction_status_history.StatusHistoryCode,
			transaction_status_history.TransactionId,
			transaction_status_history.IdPreviousStatus,
			transaction_status_history.IdNewStatus,
			transaction_status_history.Reason,
			transaction_status_history.IdChangedBy,
			transaction_status_history.GatewayResponse,
			transaction_status_history.CreatedAt,
			transaction_status_history.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Transaction_status_historyRepository.repository.UpdateTransaction_status_history.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateTransaction_status_history.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateTransaction_status_history.rows_affected", rowsAffected)
	return nil
}

