package account_typesRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Account_typesRepositoryIF interface {
     GetAccount_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetAccount_typesById(ctx context.Context, id int64) (*model.Account_types, error)
     GetAccount_typesByTypeCode(ctx context.Context, typecode string) (*model.Account_types, error)
     InsertAccount_types(ctx context.Context, account_types *model.Account_types) (int64, error)
     UpdateAccount_types(ctx context.Context, account_types *model.Account_types, id int64) error
     DeleteAccount_typesById(ctx context.Context, id int64) (bool, error)
}
 type Account_typesRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewAccount_typesRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Account_typesRepository{
    return &Account_typesRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Account_types"),
     }
}
func (t Account_typesRepository)  GetAccount_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Account_typesRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAccount_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAccount_types.offset", offset)
	tracker.AddParam("repository.GetAccount_types.limit", limit)
	itemsPage 			= model.ItemsPage{}
	account_typess := []model.Account_types{}

	rows, err := t.PGRead.Query(ctx, SQL_ACCOUNT_TYPES_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Account_typesRepository.repository.GetAccount_typess.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var account_types model.Account_types
		err := rows.Scan(
			&account_types.ID,
			&account_types.TypeCode,
			&account_types.Name,
			&account_types.Description,
			&account_types.RequiresVerification,
			&account_types.IsActive,
			&account_types.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Account_typesRepository.repository.GetAccount_typess.Scan: ", err.Error())
			return itemsPage, err
		}
		account_typess = append(account_typess, account_types)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Account_typesRepository.repository.GetAccount_typess.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(account_typess) > 0 {
		qtyRecords = account_typess[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = account_typess

	tracker.AddResult("repository.GetAccount_types.rows_returned", len(account_typess))
	tracker.AddResult("repository.GetAccount_types.total_count", len(account_typess))

	return itemsPage, nil
}
func (t Account_typesRepository)  GetAccount_typesById(ctx context.Context, id int64) (account_types *model.Account_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Account_typesRepository -> GetAccount_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAccount_typesById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAccount_typesById.id", id)

	account_types = new(model.Account_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_ACCOUNT_TYPES_BY_ID, id)
		err = row.Scan(
			&account_types.ID,
			&account_types.TypeCode,
			&account_types.Name,
			&account_types.Description,
			&account_types.RequiresVerification,
			&account_types.IsActive,
			&account_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Account_typesRepository.repository.GetAccount_typesById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetAccount_typesById.found", true)
	return account_types, nil
}
func (t Account_typesRepository)  GetAccount_typesByTypeCode(ctx context.Context, typecode string) (account_types *model.Account_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Account_typesRepository -> GetAccount_typesByTypeCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAccount_typesByTypeCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAccount_typesByTypeCode.typecode", typecode)

	account_types = new(model.Account_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_ACCOUNT_TYPES_BY_TYPE_CODE, typecode)
		err = row.Scan(
			&account_types.ID,
			&account_types.TypeCode,
			&account_types.Name,
			&account_types.Description,
			&account_types.RequiresVerification,
			&account_types.IsActive,
			&account_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Account_typesRepository.repository.GetAccount_typesBytypecode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return account_types, nil
}
func (t Account_typesRepository)  DeleteAccount_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Account_typesRepository -> DeleteAccount_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteAccount_typesById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_ACCOUNT_TYPES_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Account_typesRepository.repository.DeleteAccount_typesById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteAccount_typesById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteAccount_typesById.deleted", result)
	return true, err
}
func (t Account_typesRepository)  InsertAccount_types(ctx context.Context,account_types *model.Account_types) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Account_typesRepository -> InsertAccount_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertAccount_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertAccount_types.typecode", account_types.TypeCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_ACCOUNT_TYPES_INSERT,
			account_types.TypeCode,
			account_types.Name,
			account_types.Description,
			account_types.RequiresVerification,
			account_types.IsActive,
			account_types.CreatedAt,
	).Scan(&account_types.ID)

	if err != nil {
		t.log.Error(ctx, "Account_typesRepository.repository.InsertAccount_types.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertAccount_types.inserted_id", account_types.ID)
   return account_types.ID, nil

}
func (t Account_typesRepository)  UpdateAccount_types(ctx context.Context,account_types *model.Account_types, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Account_typesRepository -> UpdateAccount_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateAccount_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateAccount_types.id", id)
	tracker.AddParam("repository.UpdateAccount_types.typecode", account_types.TypeCode)

	account_types.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_ACCOUNT_TYPES_UPDATE, 
			account_types.TypeCode,
			account_types.Name,
			account_types.Description,
			account_types.RequiresVerification,
			account_types.IsActive,
			account_types.CreatedAt,
			account_types.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Account_typesRepository.repository.UpdateAccount_types.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateAccount_types.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateAccount_types.rows_affected", rowsAffected)
	return nil
}

