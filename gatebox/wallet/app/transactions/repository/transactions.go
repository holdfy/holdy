package transactionsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type TransactionsRepositoryIF interface {
     GetTransactions(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetTransactionsById(ctx context.Context, id int64) (*model.Transactions, error)
     GetTransactionsByTransactionCode(ctx context.Context, transactioncode string) (*model.Transactions, error)
     InsertTransactions(ctx context.Context, transactions *model.Transactions) (int64, error)
     UpdateTransactions(ctx context.Context, transactions *model.Transactions, id int64) error
     DeleteTransactionsById(ctx context.Context, id int64) (bool, error)
}
 type TransactionsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewTransactionsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *TransactionsRepository{
    return &TransactionsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Transactions"),
     }
}
func (t TransactionsRepository)  GetTransactions(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("TransactionsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetTransactions")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetTransactions.offset", offset)
	tracker.AddParam("repository.GetTransactions.limit", limit)
	itemsPage 			= model.ItemsPage{}
	transactionss := []model.Transactions{}

	rows, err := t.PGRead.Query(ctx, SQL_TRANSACTIONS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "TransactionsRepository.repository.GetTransactionss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var transactions model.Transactions
		err := rows.Scan(
			&transactions.ID,
			&transactions.TransactionCode,
			&transactions.WalletId,
			&transactions.ExternalTransactionId,
			&transactions.IdTransactionType,
			&transactions.IdPaymentMethod,
			&transactions.IdStatus,
			&transactions.Amount,
			&transactions.IdCurrency,
			&transactions.FeeAmount,
			&transactions.NetAmount,
			&transactions.PayerUserId,
			&transactions.PayerWalletId,
			&transactions.PayeeUserId,
			&transactions.PayeeWalletId,
			&transactions.PayeeExternalAccount,
			&transactions.PaymentMethodId,
			&transactions.MerchantId,
			&transactions.MerchantName,
			&transactions.MerchantCategory,
			&transactions.DeviceId,
			&transactions.DeviceInfo,
			&transactions.LocationData,
			&transactions.IpAddress,
			&transactions.RequiresSignature,
			&transactions.SignatureProvided,
			&transactions.IdSignatureMethod,
			&transactions.AuthorizedAt,
			&transactions.CompletedAt,
			&transactions.CancelledAt,
			&transactions.Description,
			&transactions.Metadata,
			&transactions.CreatedAt,
			&transactions.UpdatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "TransactionsRepository.repository.GetTransactionss.Scan: ", err.Error())
			return itemsPage, err
		}
		transactionss = append(transactionss, transactions)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "TransactionsRepository.repository.GetTransactionss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(transactionss) > 0 {
		qtyRecords = transactionss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = transactionss

	tracker.AddResult("repository.GetTransactions.rows_returned", len(transactionss))
	tracker.AddResult("repository.GetTransactions.total_count", len(transactionss))

	return itemsPage, nil
}
func (t TransactionsRepository)  GetTransactionsById(ctx context.Context, id int64) (transactions *model.Transactions, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("TransactionsRepository -> GetTransactionsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetTransactionsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetTransactionsById.id", id)

	transactions = new(model.Transactions)
	row := t.PGRead.QueryRow(ctx, SQL_GET_TRANSACTIONS_BY_ID, id)
		err = row.Scan(
			&transactions.ID,
			&transactions.TransactionCode,
			&transactions.WalletId,
			&transactions.ExternalTransactionId,
			&transactions.IdTransactionType,
			&transactions.IdPaymentMethod,
			&transactions.IdStatus,
			&transactions.Amount,
			&transactions.IdCurrency,
			&transactions.FeeAmount,
			&transactions.NetAmount,
			&transactions.PayerUserId,
			&transactions.PayerWalletId,
			&transactions.PayeeUserId,
			&transactions.PayeeWalletId,
			&transactions.PayeeExternalAccount,
			&transactions.PaymentMethodId,
			&transactions.MerchantId,
			&transactions.MerchantName,
			&transactions.MerchantCategory,
			&transactions.DeviceId,
			&transactions.DeviceInfo,
			&transactions.LocationData,
			&transactions.IpAddress,
			&transactions.RequiresSignature,
			&transactions.SignatureProvided,
			&transactions.IdSignatureMethod,
			&transactions.AuthorizedAt,
			&transactions.CompletedAt,
			&transactions.CancelledAt,
			&transactions.Description,
			&transactions.Metadata,
			&transactions.CreatedAt,
			&transactions.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"TransactionsRepository.repository.GetTransactionsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetTransactionsById.found", true)
	return transactions, nil
}
func (t TransactionsRepository)  GetTransactionsByTransactionCode(ctx context.Context, transactioncode string) (transactions *model.Transactions, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("TransactionsRepository -> GetTransactionsByTransactionCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetTransactionsByTransactionCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetTransactionsByTransactionCode.transactioncode", transactioncode)

	transactions = new(model.Transactions)
	row := t.PGRead.QueryRow(ctx, SQL_GET_TRANSACTIONS_BY_TRANSACTION_CODE, transactioncode)
		err = row.Scan(
			&transactions.ID,
			&transactions.TransactionCode,
			&transactions.WalletId,
			&transactions.ExternalTransactionId,
			&transactions.IdTransactionType,
			&transactions.IdPaymentMethod,
			&transactions.IdStatus,
			&transactions.Amount,
			&transactions.IdCurrency,
			&transactions.FeeAmount,
			&transactions.NetAmount,
			&transactions.PayerUserId,
			&transactions.PayerWalletId,
			&transactions.PayeeUserId,
			&transactions.PayeeWalletId,
			&transactions.PayeeExternalAccount,
			&transactions.PaymentMethodId,
			&transactions.MerchantId,
			&transactions.MerchantName,
			&transactions.MerchantCategory,
			&transactions.DeviceId,
			&transactions.DeviceInfo,
			&transactions.LocationData,
			&transactions.IpAddress,
			&transactions.RequiresSignature,
			&transactions.SignatureProvided,
			&transactions.IdSignatureMethod,
			&transactions.AuthorizedAt,
			&transactions.CompletedAt,
			&transactions.CancelledAt,
			&transactions.Description,
			&transactions.Metadata,
			&transactions.CreatedAt,
			&transactions.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"TransactionsRepository.repository.GetTransactionsBytransactioncode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return transactions, nil
}
func (t TransactionsRepository)  DeleteTransactionsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("TransactionsRepository -> DeleteTransactionsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteTransactionsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_TRANSACTIONS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"TransactionsRepository.repository.DeleteTransactionsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteTransactionsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteTransactionsById.deleted", result)
	return true, err
}
func (t TransactionsRepository)  InsertTransactions(ctx context.Context,transactions *model.Transactions) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("TransactionsRepository -> InsertTransactions", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertTransactions")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertTransactions.transactioncode", transactions.TransactionCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_TRANSACTIONS_INSERT,
			transactions.TransactionCode,
			transactions.WalletId,
			transactions.ExternalTransactionId,
			transactions.IdTransactionType,
			transactions.IdPaymentMethod,
			transactions.IdStatus,
			transactions.Amount,
			transactions.IdCurrency,
			transactions.FeeAmount,
			transactions.NetAmount,
			transactions.PayerUserId,
			transactions.PayerWalletId,
			transactions.PayeeUserId,
			transactions.PayeeWalletId,
			transactions.PayeeExternalAccount,
			transactions.PaymentMethodId,
			transactions.MerchantId,
			transactions.MerchantName,
			transactions.MerchantCategory,
			transactions.DeviceId,
			transactions.DeviceInfo,
			transactions.LocationData,
			transactions.IpAddress,
			transactions.RequiresSignature,
			transactions.SignatureProvided,
			transactions.IdSignatureMethod,
			transactions.AuthorizedAt,
			transactions.CompletedAt,
			transactions.CancelledAt,
			transactions.Description,
			transactions.Metadata,
			transactions.CreatedAt,
			transactions.UpdatedAt,
	).Scan(&transactions.ID)

	if err != nil {
		t.log.Error(ctx, "TransactionsRepository.repository.InsertTransactions.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertTransactions.inserted_id", transactions.ID)
   return transactions.ID, nil

}
func (t TransactionsRepository)  UpdateTransactions(ctx context.Context,transactions *model.Transactions, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("TransactionsRepository -> UpdateTransactions", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateTransactions")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateTransactions.id", id)
	tracker.AddParam("repository.UpdateTransactions.transactioncode", transactions.TransactionCode)

	transactions.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_TRANSACTIONS_UPDATE, 
			transactions.TransactionCode,
			transactions.WalletId,
			transactions.ExternalTransactionId,
			transactions.IdTransactionType,
			transactions.IdPaymentMethod,
			transactions.IdStatus,
			transactions.Amount,
			transactions.IdCurrency,
			transactions.FeeAmount,
			transactions.NetAmount,
			transactions.PayerUserId,
			transactions.PayerWalletId,
			transactions.PayeeUserId,
			transactions.PayeeWalletId,
			transactions.PayeeExternalAccount,
			transactions.PaymentMethodId,
			transactions.MerchantId,
			transactions.MerchantName,
			transactions.MerchantCategory,
			transactions.DeviceId,
			transactions.DeviceInfo,
			transactions.LocationData,
			transactions.IpAddress,
			transactions.RequiresSignature,
			transactions.SignatureProvided,
			transactions.IdSignatureMethod,
			transactions.AuthorizedAt,
			transactions.CompletedAt,
			transactions.CancelledAt,
			transactions.Description,
			transactions.Metadata,
			transactions.CreatedAt,
			transactions.UpdatedAt,
			transactions.ID,
   )
	if err != nil {
		t.log.Error(ctx, "TransactionsRepository.repository.UpdateTransactions.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateTransactions.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateTransactions.rows_affected", rowsAffected)
	return nil
}

