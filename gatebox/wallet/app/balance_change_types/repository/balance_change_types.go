package balance_change_typesRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Balance_change_typesRepositoryIF interface {
     GetBalance_change_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetBalance_change_typesById(ctx context.Context, id int64) (*model.Balance_change_types, error)
     GetBalance_change_typesByTypeCode(ctx context.Context, typecode string) (*model.Balance_change_types, error)
     InsertBalance_change_types(ctx context.Context, balance_change_types *model.Balance_change_types) (int64, error)
     UpdateBalance_change_types(ctx context.Context, balance_change_types *model.Balance_change_types, id int64) error
     DeleteBalance_change_typesById(ctx context.Context, id int64) (bool, error)
}
 type Balance_change_typesRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewBalance_change_typesRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Balance_change_typesRepository{
    return &Balance_change_typesRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Balance_change_types"),
     }
}
func (t Balance_change_typesRepository)  GetBalance_change_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Balance_change_typesRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetBalance_change_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetBalance_change_types.offset", offset)
	tracker.AddParam("repository.GetBalance_change_types.limit", limit)
	itemsPage 			= model.ItemsPage{}
	balance_change_typess := []model.Balance_change_types{}

	rows, err := t.PGRead.Query(ctx, SQL_BALANCE_CHANGE_TYPES_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Balance_change_typesRepository.repository.GetBalance_change_typess.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var balance_change_types model.Balance_change_types
		err := rows.Scan(
			&balance_change_types.ID,
			&balance_change_types.TypeCode,
			&balance_change_types.Name,
			&balance_change_types.Description,
			&balance_change_types.IsPositive,
			&balance_change_types.IsActive,
			&balance_change_types.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Balance_change_typesRepository.repository.GetBalance_change_typess.Scan: ", err.Error())
			return itemsPage, err
		}
		balance_change_typess = append(balance_change_typess, balance_change_types)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Balance_change_typesRepository.repository.GetBalance_change_typess.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(balance_change_typess) > 0 {
		qtyRecords = balance_change_typess[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = balance_change_typess

	tracker.AddResult("repository.GetBalance_change_types.rows_returned", len(balance_change_typess))
	tracker.AddResult("repository.GetBalance_change_types.total_count", len(balance_change_typess))

	return itemsPage, nil
}
func (t Balance_change_typesRepository)  GetBalance_change_typesById(ctx context.Context, id int64) (balance_change_types *model.Balance_change_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Balance_change_typesRepository -> GetBalance_change_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetBalance_change_typesById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetBalance_change_typesById.id", id)

	balance_change_types = new(model.Balance_change_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_BALANCE_CHANGE_TYPES_BY_ID, id)
		err = row.Scan(
			&balance_change_types.ID,
			&balance_change_types.TypeCode,
			&balance_change_types.Name,
			&balance_change_types.Description,
			&balance_change_types.IsPositive,
			&balance_change_types.IsActive,
			&balance_change_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Balance_change_typesRepository.repository.GetBalance_change_typesById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetBalance_change_typesById.found", true)
	return balance_change_types, nil
}
func (t Balance_change_typesRepository)  GetBalance_change_typesByTypeCode(ctx context.Context, typecode string) (balance_change_types *model.Balance_change_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Balance_change_typesRepository -> GetBalance_change_typesByTypeCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetBalance_change_typesByTypeCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetBalance_change_typesByTypeCode.typecode", typecode)

	balance_change_types = new(model.Balance_change_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_BALANCE_CHANGE_TYPES_BY_TYPE_CODE, typecode)
		err = row.Scan(
			&balance_change_types.ID,
			&balance_change_types.TypeCode,
			&balance_change_types.Name,
			&balance_change_types.Description,
			&balance_change_types.IsPositive,
			&balance_change_types.IsActive,
			&balance_change_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Balance_change_typesRepository.repository.GetBalance_change_typesBytypecode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return balance_change_types, nil
}
func (t Balance_change_typesRepository)  DeleteBalance_change_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Balance_change_typesRepository -> DeleteBalance_change_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteBalance_change_typesById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_BALANCE_CHANGE_TYPES_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Balance_change_typesRepository.repository.DeleteBalance_change_typesById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteBalance_change_typesById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteBalance_change_typesById.deleted", result)
	return true, err
}
func (t Balance_change_typesRepository)  InsertBalance_change_types(ctx context.Context,balance_change_types *model.Balance_change_types) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Balance_change_typesRepository -> InsertBalance_change_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertBalance_change_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertBalance_change_types.typecode", balance_change_types.TypeCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_BALANCE_CHANGE_TYPES_INSERT,
			balance_change_types.TypeCode,
			balance_change_types.Name,
			balance_change_types.Description,
			balance_change_types.IsPositive,
			balance_change_types.IsActive,
			balance_change_types.CreatedAt,
	).Scan(&balance_change_types.ID)

	if err != nil {
		t.log.Error(ctx, "Balance_change_typesRepository.repository.InsertBalance_change_types.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertBalance_change_types.inserted_id", balance_change_types.ID)
   return balance_change_types.ID, nil

}
func (t Balance_change_typesRepository)  UpdateBalance_change_types(ctx context.Context,balance_change_types *model.Balance_change_types, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Balance_change_typesRepository -> UpdateBalance_change_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateBalance_change_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateBalance_change_types.id", id)
	tracker.AddParam("repository.UpdateBalance_change_types.typecode", balance_change_types.TypeCode)

	balance_change_types.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_BALANCE_CHANGE_TYPES_UPDATE, 
			balance_change_types.TypeCode,
			balance_change_types.Name,
			balance_change_types.Description,
			balance_change_types.IsPositive,
			balance_change_types.IsActive,
			balance_change_types.CreatedAt,
			balance_change_types.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Balance_change_typesRepository.repository.UpdateBalance_change_types.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateBalance_change_types.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateBalance_change_types.rows_affected", rowsAffected)
	return nil
}

