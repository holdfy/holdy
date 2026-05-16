package address_typesRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Address_typesRepositoryIF interface {
     GetAddress_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetAddress_typesById(ctx context.Context, id int64) (*model.Address_types, error)
     GetAddress_typesByTypeCode(ctx context.Context, typecode string) (*model.Address_types, error)
     InsertAddress_types(ctx context.Context, address_types *model.Address_types) (int64, error)
     UpdateAddress_types(ctx context.Context, address_types *model.Address_types, id int64) error
     DeleteAddress_typesById(ctx context.Context, id int64) (bool, error)
}
 type Address_typesRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewAddress_typesRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Address_typesRepository{
    return &Address_typesRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Address_types"),
     }
}
func (t Address_typesRepository)  GetAddress_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Address_typesRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAddress_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAddress_types.offset", offset)
	tracker.AddParam("repository.GetAddress_types.limit", limit)
	itemsPage 			= model.ItemsPage{}
	address_typess := []model.Address_types{}

	rows, err := t.PGRead.Query(ctx, SQL_ADDRESS_TYPES_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Address_typesRepository.repository.GetAddress_typess.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var address_types model.Address_types
		err := rows.Scan(
			&address_types.ID,
			&address_types.TypeCode,
			&address_types.Name,
			&address_types.Description,
			&address_types.IsActive,
			&address_types.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Address_typesRepository.repository.GetAddress_typess.Scan: ", err.Error())
			return itemsPage, err
		}
		address_typess = append(address_typess, address_types)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Address_typesRepository.repository.GetAddress_typess.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(address_typess) > 0 {
		qtyRecords = address_typess[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = address_typess

	tracker.AddResult("repository.GetAddress_types.rows_returned", len(address_typess))
	tracker.AddResult("repository.GetAddress_types.total_count", len(address_typess))

	return itemsPage, nil
}
func (t Address_typesRepository)  GetAddress_typesById(ctx context.Context, id int64) (address_types *model.Address_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Address_typesRepository -> GetAddress_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAddress_typesById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAddress_typesById.id", id)

	address_types = new(model.Address_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_ADDRESS_TYPES_BY_ID, id)
		err = row.Scan(
			&address_types.ID,
			&address_types.TypeCode,
			&address_types.Name,
			&address_types.Description,
			&address_types.IsActive,
			&address_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Address_typesRepository.repository.GetAddress_typesById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetAddress_typesById.found", true)
	return address_types, nil
}
func (t Address_typesRepository)  GetAddress_typesByTypeCode(ctx context.Context, typecode string) (address_types *model.Address_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Address_typesRepository -> GetAddress_typesByTypeCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAddress_typesByTypeCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAddress_typesByTypeCode.typecode", typecode)

	address_types = new(model.Address_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_ADDRESS_TYPES_BY_TYPE_CODE, typecode)
		err = row.Scan(
			&address_types.ID,
			&address_types.TypeCode,
			&address_types.Name,
			&address_types.Description,
			&address_types.IsActive,
			&address_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Address_typesRepository.repository.GetAddress_typesBytypecode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return address_types, nil
}
func (t Address_typesRepository)  DeleteAddress_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Address_typesRepository -> DeleteAddress_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteAddress_typesById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_ADDRESS_TYPES_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Address_typesRepository.repository.DeleteAddress_typesById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteAddress_typesById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteAddress_typesById.deleted", result)
	return true, err
}
func (t Address_typesRepository)  InsertAddress_types(ctx context.Context,address_types *model.Address_types) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Address_typesRepository -> InsertAddress_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertAddress_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertAddress_types.typecode", address_types.TypeCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_ADDRESS_TYPES_INSERT,
			address_types.TypeCode,
			address_types.Name,
			address_types.Description,
			address_types.IsActive,
			address_types.CreatedAt,
	).Scan(&address_types.ID)

	if err != nil {
		t.log.Error(ctx, "Address_typesRepository.repository.InsertAddress_types.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertAddress_types.inserted_id", address_types.ID)
   return address_types.ID, nil

}
func (t Address_typesRepository)  UpdateAddress_types(ctx context.Context,address_types *model.Address_types, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Address_typesRepository -> UpdateAddress_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateAddress_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateAddress_types.id", id)
	tracker.AddParam("repository.UpdateAddress_types.typecode", address_types.TypeCode)

	address_types.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_ADDRESS_TYPES_UPDATE, 
			address_types.TypeCode,
			address_types.Name,
			address_types.Description,
			address_types.IsActive,
			address_types.CreatedAt,
			address_types.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Address_typesRepository.repository.UpdateAddress_types.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateAddress_types.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateAddress_types.rows_affected", rowsAffected)
	return nil
}

