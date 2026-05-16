package wallet_typesRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Wallet_typesRepositoryIF interface {
     GetWallet_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetWallet_typesById(ctx context.Context, id int64) (*model.Wallet_types, error)
     GetWallet_typesByTypeCode(ctx context.Context, typecode string) (*model.Wallet_types, error)
     InsertWallet_types(ctx context.Context, wallet_types *model.Wallet_types) (int64, error)
     UpdateWallet_types(ctx context.Context, wallet_types *model.Wallet_types, id int64) error
     DeleteWallet_typesById(ctx context.Context, id int64) (bool, error)
}
 type Wallet_typesRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewWallet_typesRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Wallet_typesRepository{
    return &Wallet_typesRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Wallet_types"),
     }
}
func (t Wallet_typesRepository)  GetWallet_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_typesRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetWallet_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetWallet_types.offset", offset)
	tracker.AddParam("repository.GetWallet_types.limit", limit)
	itemsPage 			= model.ItemsPage{}
	wallet_typess := []model.Wallet_types{}

	rows, err := t.PGRead.Query(ctx, SQL_WALLET_TYPES_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Wallet_typesRepository.repository.GetWallet_typess.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var wallet_types model.Wallet_types
		err := rows.Scan(
			&wallet_types.ID,
			&wallet_types.TypeCode,
			&wallet_types.Name,
			&wallet_types.Description,
			&wallet_types.DefaultDailyLimit,
			&wallet_types.DefaultMonthlyLimit,
			&wallet_types.IsActive,
			&wallet_types.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Wallet_typesRepository.repository.GetWallet_typess.Scan: ", err.Error())
			return itemsPage, err
		}
		wallet_typess = append(wallet_typess, wallet_types)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Wallet_typesRepository.repository.GetWallet_typess.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(wallet_typess) > 0 {
		qtyRecords = wallet_typess[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = wallet_typess

	tracker.AddResult("repository.GetWallet_types.rows_returned", len(wallet_typess))
	tracker.AddResult("repository.GetWallet_types.total_count", len(wallet_typess))

	return itemsPage, nil
}
func (t Wallet_typesRepository)  GetWallet_typesById(ctx context.Context, id int64) (wallet_types *model.Wallet_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_typesRepository -> GetWallet_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetWallet_typesById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetWallet_typesById.id", id)

	wallet_types = new(model.Wallet_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_WALLET_TYPES_BY_ID, id)
		err = row.Scan(
			&wallet_types.ID,
			&wallet_types.TypeCode,
			&wallet_types.Name,
			&wallet_types.Description,
			&wallet_types.DefaultDailyLimit,
			&wallet_types.DefaultMonthlyLimit,
			&wallet_types.IsActive,
			&wallet_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Wallet_typesRepository.repository.GetWallet_typesById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetWallet_typesById.found", true)
	return wallet_types, nil
}
func (t Wallet_typesRepository)  GetWallet_typesByTypeCode(ctx context.Context, typecode string) (wallet_types *model.Wallet_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_typesRepository -> GetWallet_typesByTypeCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetWallet_typesByTypeCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetWallet_typesByTypeCode.typecode", typecode)

	wallet_types = new(model.Wallet_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_WALLET_TYPES_BY_TYPE_CODE, typecode)
		err = row.Scan(
			&wallet_types.ID,
			&wallet_types.TypeCode,
			&wallet_types.Name,
			&wallet_types.Description,
			&wallet_types.DefaultDailyLimit,
			&wallet_types.DefaultMonthlyLimit,
			&wallet_types.IsActive,
			&wallet_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Wallet_typesRepository.repository.GetWallet_typesBytypecode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return wallet_types, nil
}
func (t Wallet_typesRepository)  DeleteWallet_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_typesRepository -> DeleteWallet_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteWallet_typesById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_WALLET_TYPES_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Wallet_typesRepository.repository.DeleteWallet_typesById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteWallet_typesById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteWallet_typesById.deleted", result)
	return true, err
}
func (t Wallet_typesRepository)  InsertWallet_types(ctx context.Context,wallet_types *model.Wallet_types) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_typesRepository -> InsertWallet_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertWallet_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertWallet_types.typecode", wallet_types.TypeCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_WALLET_TYPES_INSERT,
			wallet_types.TypeCode,
			wallet_types.Name,
			wallet_types.Description,
			wallet_types.DefaultDailyLimit,
			wallet_types.DefaultMonthlyLimit,
			wallet_types.IsActive,
			wallet_types.CreatedAt,
	).Scan(&wallet_types.ID)

	if err != nil {
		t.log.Error(ctx, "Wallet_typesRepository.repository.InsertWallet_types.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertWallet_types.inserted_id", wallet_types.ID)
   return wallet_types.ID, nil

}
func (t Wallet_typesRepository)  UpdateWallet_types(ctx context.Context,wallet_types *model.Wallet_types, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Wallet_typesRepository -> UpdateWallet_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateWallet_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateWallet_types.id", id)
	tracker.AddParam("repository.UpdateWallet_types.typecode", wallet_types.TypeCode)

	wallet_types.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_WALLET_TYPES_UPDATE, 
			wallet_types.TypeCode,
			wallet_types.Name,
			wallet_types.Description,
			wallet_types.DefaultDailyLimit,
			wallet_types.DefaultMonthlyLimit,
			wallet_types.IsActive,
			wallet_types.CreatedAt,
			wallet_types.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Wallet_typesRepository.repository.UpdateWallet_types.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateWallet_types.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateWallet_types.rows_affected", rowsAffected)
	return nil
}

