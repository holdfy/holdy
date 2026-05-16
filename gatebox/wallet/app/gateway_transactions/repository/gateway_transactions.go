package gateway_transactionsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Gateway_transactionsRepositoryIF interface {
     GetGateway_transactions(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetGateway_transactionsById(ctx context.Context, id int64) (*model.Gateway_transactions, error)
     GetGateway_transactionsByGatewayTransactionCode(ctx context.Context, gatewaytransactioncode string) (*model.Gateway_transactions, error)
     InsertGateway_transactions(ctx context.Context, gateway_transactions *model.Gateway_transactions) (int64, error)
     UpdateGateway_transactions(ctx context.Context, gateway_transactions *model.Gateway_transactions, id int64) error
     DeleteGateway_transactionsById(ctx context.Context, id int64) (bool, error)
}
 type Gateway_transactionsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewGateway_transactionsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Gateway_transactionsRepository{
    return &Gateway_transactionsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Gateway_transactions"),
     }
}
func (t Gateway_transactionsRepository)  GetGateway_transactions(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Gateway_transactionsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetGateway_transactions")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetGateway_transactions.offset", offset)
	tracker.AddParam("repository.GetGateway_transactions.limit", limit)
	itemsPage 			= model.ItemsPage{}
	gateway_transactionss := []model.Gateway_transactions{}

	rows, err := t.PGRead.Query(ctx, SQL_GATEWAY_TRANSACTIONS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Gateway_transactionsRepository.repository.GetGateway_transactionss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var gateway_transactions model.Gateway_transactions
		err := rows.Scan(
			&gateway_transactions.ID,
			&gateway_transactions.GatewayTransactionCode,
			&gateway_transactions.TransactionId,
			&gateway_transactions.IdGateway,
			&gateway_transactions.GatewayTransactionId,
			&gateway_transactions.IdGatewayStatus,
			&gateway_transactions.GatewayResponse,
			&gateway_transactions.GatewayRequest,
			&gateway_transactions.ProcessingTimeMs,
			&gateway_transactions.RetryCount,
			&gateway_transactions.LastRetryAt,
			&gateway_transactions.CreatedAt,
			&gateway_transactions.UpdatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Gateway_transactionsRepository.repository.GetGateway_transactionss.Scan: ", err.Error())
			return itemsPage, err
		}
		gateway_transactionss = append(gateway_transactionss, gateway_transactions)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Gateway_transactionsRepository.repository.GetGateway_transactionss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(gateway_transactionss) > 0 {
		qtyRecords = gateway_transactionss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = gateway_transactionss

	tracker.AddResult("repository.GetGateway_transactions.rows_returned", len(gateway_transactionss))
	tracker.AddResult("repository.GetGateway_transactions.total_count", len(gateway_transactionss))

	return itemsPage, nil
}
func (t Gateway_transactionsRepository)  GetGateway_transactionsById(ctx context.Context, id int64) (gateway_transactions *model.Gateway_transactions, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Gateway_transactionsRepository -> GetGateway_transactionsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetGateway_transactionsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetGateway_transactionsById.id", id)

	gateway_transactions = new(model.Gateway_transactions)
	row := t.PGRead.QueryRow(ctx, SQL_GET_GATEWAY_TRANSACTIONS_BY_ID, id)
		err = row.Scan(
			&gateway_transactions.ID,
			&gateway_transactions.GatewayTransactionCode,
			&gateway_transactions.TransactionId,
			&gateway_transactions.IdGateway,
			&gateway_transactions.GatewayTransactionId,
			&gateway_transactions.IdGatewayStatus,
			&gateway_transactions.GatewayResponse,
			&gateway_transactions.GatewayRequest,
			&gateway_transactions.ProcessingTimeMs,
			&gateway_transactions.RetryCount,
			&gateway_transactions.LastRetryAt,
			&gateway_transactions.CreatedAt,
			&gateway_transactions.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Gateway_transactionsRepository.repository.GetGateway_transactionsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetGateway_transactionsById.found", true)
	return gateway_transactions, nil
}
func (t Gateway_transactionsRepository)  GetGateway_transactionsByGatewayTransactionCode(ctx context.Context, gatewaytransactioncode string) (gateway_transactions *model.Gateway_transactions, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Gateway_transactionsRepository -> GetGateway_transactionsByGatewayTransactionCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetGateway_transactionsByGatewayTransactionCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetGateway_transactionsByGatewayTransactionCode.gatewaytransactioncode", gatewaytransactioncode)

	gateway_transactions = new(model.Gateway_transactions)
	row := t.PGRead.QueryRow(ctx, SQL_GET_GATEWAY_TRANSACTIONS_BY_GATEWAY_TRANSACTION_CODE, gatewaytransactioncode)
		err = row.Scan(
			&gateway_transactions.ID,
			&gateway_transactions.GatewayTransactionCode,
			&gateway_transactions.TransactionId,
			&gateway_transactions.IdGateway,
			&gateway_transactions.GatewayTransactionId,
			&gateway_transactions.IdGatewayStatus,
			&gateway_transactions.GatewayResponse,
			&gateway_transactions.GatewayRequest,
			&gateway_transactions.ProcessingTimeMs,
			&gateway_transactions.RetryCount,
			&gateway_transactions.LastRetryAt,
			&gateway_transactions.CreatedAt,
			&gateway_transactions.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Gateway_transactionsRepository.repository.GetGateway_transactionsBygatewaytransactioncode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return gateway_transactions, nil
}
func (t Gateway_transactionsRepository)  DeleteGateway_transactionsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Gateway_transactionsRepository -> DeleteGateway_transactionsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteGateway_transactionsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_GATEWAY_TRANSACTIONS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Gateway_transactionsRepository.repository.DeleteGateway_transactionsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteGateway_transactionsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteGateway_transactionsById.deleted", result)
	return true, err
}
func (t Gateway_transactionsRepository)  InsertGateway_transactions(ctx context.Context,gateway_transactions *model.Gateway_transactions) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Gateway_transactionsRepository -> InsertGateway_transactions", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertGateway_transactions")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertGateway_transactions.gatewaytransactioncode", gateway_transactions.GatewayTransactionCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_GATEWAY_TRANSACTIONS_INSERT,
			gateway_transactions.GatewayTransactionCode,
			gateway_transactions.TransactionId,
			gateway_transactions.IdGateway,
			gateway_transactions.GatewayTransactionId,
			gateway_transactions.IdGatewayStatus,
			gateway_transactions.GatewayResponse,
			gateway_transactions.GatewayRequest,
			gateway_transactions.ProcessingTimeMs,
			gateway_transactions.RetryCount,
			gateway_transactions.LastRetryAt,
			gateway_transactions.CreatedAt,
			gateway_transactions.UpdatedAt,
	).Scan(&gateway_transactions.ID)

	if err != nil {
		t.log.Error(ctx, "Gateway_transactionsRepository.repository.InsertGateway_transactions.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertGateway_transactions.inserted_id", gateway_transactions.ID)
   return gateway_transactions.ID, nil

}
func (t Gateway_transactionsRepository)  UpdateGateway_transactions(ctx context.Context,gateway_transactions *model.Gateway_transactions, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Gateway_transactionsRepository -> UpdateGateway_transactions", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateGateway_transactions")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateGateway_transactions.id", id)
	tracker.AddParam("repository.UpdateGateway_transactions.gatewaytransactioncode", gateway_transactions.GatewayTransactionCode)

	gateway_transactions.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_GATEWAY_TRANSACTIONS_UPDATE, 
			gateway_transactions.GatewayTransactionCode,
			gateway_transactions.TransactionId,
			gateway_transactions.IdGateway,
			gateway_transactions.GatewayTransactionId,
			gateway_transactions.IdGatewayStatus,
			gateway_transactions.GatewayResponse,
			gateway_transactions.GatewayRequest,
			gateway_transactions.ProcessingTimeMs,
			gateway_transactions.RetryCount,
			gateway_transactions.LastRetryAt,
			gateway_transactions.CreatedAt,
			gateway_transactions.UpdatedAt,
			gateway_transactions.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Gateway_transactionsRepository.repository.UpdateGateway_transactions.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateGateway_transactions.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateGateway_transactions.rows_affected", rowsAffected)
	return nil
}

