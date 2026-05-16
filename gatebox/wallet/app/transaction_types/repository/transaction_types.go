package transaction_typesRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Transaction_typesRepositoryIF interface {
     GetTransaction_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetTransaction_typesById(ctx context.Context, id int64) (*model.Transaction_types, error)
     GetTransaction_typesByTypeCode(ctx context.Context, typecode string) (*model.Transaction_types, error)
     InsertTransaction_types(ctx context.Context, transaction_types *model.Transaction_types) (int64, error)
     UpdateTransaction_types(ctx context.Context, transaction_types *model.Transaction_types, id int64) error
     DeleteTransaction_typesById(ctx context.Context, id int64) (bool, error)
}
 type Transaction_typesRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewTransaction_typesRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Transaction_typesRepository{
    return &Transaction_typesRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Transaction_types"),
     }
}
func (t Transaction_typesRepository)  GetTransaction_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_typesRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetTransaction_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetTransaction_types.offset", offset)
	tracker.AddParam("repository.GetTransaction_types.limit", limit)
	itemsPage 			= model.ItemsPage{}
	transaction_typess := []model.Transaction_types{}

	rows, err := t.PGRead.Query(ctx, SQL_TRANSACTION_TYPES_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Transaction_typesRepository.repository.GetTransaction_typess.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var transaction_types model.Transaction_types
		err := rows.Scan(
			&transaction_types.ID,
			&transaction_types.TypeCode,
			&transaction_types.Name,
			&transaction_types.Description,
			&transaction_types.AffectsBalance,
			&transaction_types.RequiresRecipient,
			&transaction_types.IsActive,
			&transaction_types.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Transaction_typesRepository.repository.GetTransaction_typess.Scan: ", err.Error())
			return itemsPage, err
		}
		transaction_typess = append(transaction_typess, transaction_types)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Transaction_typesRepository.repository.GetTransaction_typess.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(transaction_typess) > 0 {
		qtyRecords = transaction_typess[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = transaction_typess

	tracker.AddResult("repository.GetTransaction_types.rows_returned", len(transaction_typess))
	tracker.AddResult("repository.GetTransaction_types.total_count", len(transaction_typess))

	return itemsPage, nil
}
func (t Transaction_typesRepository)  GetTransaction_typesById(ctx context.Context, id int64) (transaction_types *model.Transaction_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_typesRepository -> GetTransaction_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetTransaction_typesById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetTransaction_typesById.id", id)

	transaction_types = new(model.Transaction_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_TRANSACTION_TYPES_BY_ID, id)
		err = row.Scan(
			&transaction_types.ID,
			&transaction_types.TypeCode,
			&transaction_types.Name,
			&transaction_types.Description,
			&transaction_types.AffectsBalance,
			&transaction_types.RequiresRecipient,
			&transaction_types.IsActive,
			&transaction_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Transaction_typesRepository.repository.GetTransaction_typesById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetTransaction_typesById.found", true)
	return transaction_types, nil
}
func (t Transaction_typesRepository)  GetTransaction_typesByTypeCode(ctx context.Context, typecode string) (transaction_types *model.Transaction_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_typesRepository -> GetTransaction_typesByTypeCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetTransaction_typesByTypeCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetTransaction_typesByTypeCode.typecode", typecode)

	transaction_types = new(model.Transaction_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_TRANSACTION_TYPES_BY_TYPE_CODE, typecode)
		err = row.Scan(
			&transaction_types.ID,
			&transaction_types.TypeCode,
			&transaction_types.Name,
			&transaction_types.Description,
			&transaction_types.AffectsBalance,
			&transaction_types.RequiresRecipient,
			&transaction_types.IsActive,
			&transaction_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Transaction_typesRepository.repository.GetTransaction_typesBytypecode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return transaction_types, nil
}
func (t Transaction_typesRepository)  DeleteTransaction_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_typesRepository -> DeleteTransaction_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteTransaction_typesById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_TRANSACTION_TYPES_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Transaction_typesRepository.repository.DeleteTransaction_typesById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteTransaction_typesById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteTransaction_typesById.deleted", result)
	return true, err
}
func (t Transaction_typesRepository)  InsertTransaction_types(ctx context.Context,transaction_types *model.Transaction_types) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_typesRepository -> InsertTransaction_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertTransaction_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertTransaction_types.typecode", transaction_types.TypeCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_TRANSACTION_TYPES_INSERT,
			transaction_types.TypeCode,
			transaction_types.Name,
			transaction_types.Description,
			transaction_types.AffectsBalance,
			transaction_types.RequiresRecipient,
			transaction_types.IsActive,
			transaction_types.CreatedAt,
	).Scan(&transaction_types.ID)

	if err != nil {
		t.log.Error(ctx, "Transaction_typesRepository.repository.InsertTransaction_types.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertTransaction_types.inserted_id", transaction_types.ID)
   return transaction_types.ID, nil

}
func (t Transaction_typesRepository)  UpdateTransaction_types(ctx context.Context,transaction_types *model.Transaction_types, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Transaction_typesRepository -> UpdateTransaction_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateTransaction_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateTransaction_types.id", id)
	tracker.AddParam("repository.UpdateTransaction_types.typecode", transaction_types.TypeCode)

	transaction_types.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_TRANSACTION_TYPES_UPDATE, 
			transaction_types.TypeCode,
			transaction_types.Name,
			transaction_types.Description,
			transaction_types.AffectsBalance,
			transaction_types.RequiresRecipient,
			transaction_types.IsActive,
			transaction_types.CreatedAt,
			transaction_types.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Transaction_typesRepository.repository.UpdateTransaction_types.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateTransaction_types.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateTransaction_types.rows_affected", rowsAffected)
	return nil
}

