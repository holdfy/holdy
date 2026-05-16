package device_typesRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Device_typesRepositoryIF interface {
     GetDevice_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetDevice_typesById(ctx context.Context, id int64) (*model.Device_types, error)
     GetDevice_typesByTypeCode(ctx context.Context, typecode string) (*model.Device_types, error)
     InsertDevice_types(ctx context.Context, device_types *model.Device_types) (int64, error)
     UpdateDevice_types(ctx context.Context, device_types *model.Device_types, id int64) error
     DeleteDevice_typesById(ctx context.Context, id int64) (bool, error)
}
 type Device_typesRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewDevice_typesRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Device_typesRepository{
    return &Device_typesRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Device_types"),
     }
}
func (t Device_typesRepository)  GetDevice_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Device_typesRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetDevice_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetDevice_types.offset", offset)
	tracker.AddParam("repository.GetDevice_types.limit", limit)
	itemsPage 			= model.ItemsPage{}
	device_typess := []model.Device_types{}

	rows, err := t.PGRead.Query(ctx, SQL_DEVICE_TYPES_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Device_typesRepository.repository.GetDevice_typess.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var device_types model.Device_types
		err := rows.Scan(
			&device_types.ID,
			&device_types.TypeCode,
			&device_types.Name,
			&device_types.Description,
			&device_types.IsMobile,
			&device_types.SecurityLevel,
			&device_types.IsActive,
			&device_types.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Device_typesRepository.repository.GetDevice_typess.Scan: ", err.Error())
			return itemsPage, err
		}
		device_typess = append(device_typess, device_types)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Device_typesRepository.repository.GetDevice_typess.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(device_typess) > 0 {
		qtyRecords = device_typess[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = device_typess

	tracker.AddResult("repository.GetDevice_types.rows_returned", len(device_typess))
	tracker.AddResult("repository.GetDevice_types.total_count", len(device_typess))

	return itemsPage, nil
}
func (t Device_typesRepository)  GetDevice_typesById(ctx context.Context, id int64) (device_types *model.Device_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Device_typesRepository -> GetDevice_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetDevice_typesById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetDevice_typesById.id", id)

	device_types = new(model.Device_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_DEVICE_TYPES_BY_ID, id)
		err = row.Scan(
			&device_types.ID,
			&device_types.TypeCode,
			&device_types.Name,
			&device_types.Description,
			&device_types.IsMobile,
			&device_types.SecurityLevel,
			&device_types.IsActive,
			&device_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Device_typesRepository.repository.GetDevice_typesById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetDevice_typesById.found", true)
	return device_types, nil
}
func (t Device_typesRepository)  GetDevice_typesByTypeCode(ctx context.Context, typecode string) (device_types *model.Device_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Device_typesRepository -> GetDevice_typesByTypeCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetDevice_typesByTypeCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetDevice_typesByTypeCode.typecode", typecode)

	device_types = new(model.Device_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_DEVICE_TYPES_BY_TYPE_CODE, typecode)
		err = row.Scan(
			&device_types.ID,
			&device_types.TypeCode,
			&device_types.Name,
			&device_types.Description,
			&device_types.IsMobile,
			&device_types.SecurityLevel,
			&device_types.IsActive,
			&device_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Device_typesRepository.repository.GetDevice_typesBytypecode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return device_types, nil
}
func (t Device_typesRepository)  DeleteDevice_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Device_typesRepository -> DeleteDevice_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteDevice_typesById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_DEVICE_TYPES_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Device_typesRepository.repository.DeleteDevice_typesById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteDevice_typesById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteDevice_typesById.deleted", result)
	return true, err
}
func (t Device_typesRepository)  InsertDevice_types(ctx context.Context,device_types *model.Device_types) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Device_typesRepository -> InsertDevice_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertDevice_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertDevice_types.typecode", device_types.TypeCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_DEVICE_TYPES_INSERT,
			device_types.TypeCode,
			device_types.Name,
			device_types.Description,
			device_types.IsMobile,
			device_types.SecurityLevel,
			device_types.IsActive,
			device_types.CreatedAt,
	).Scan(&device_types.ID)

	if err != nil {
		t.log.Error(ctx, "Device_typesRepository.repository.InsertDevice_types.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertDevice_types.inserted_id", device_types.ID)
   return device_types.ID, nil

}
func (t Device_typesRepository)  UpdateDevice_types(ctx context.Context,device_types *model.Device_types, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Device_typesRepository -> UpdateDevice_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateDevice_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateDevice_types.id", id)
	tracker.AddParam("repository.UpdateDevice_types.typecode", device_types.TypeCode)

	device_types.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_DEVICE_TYPES_UPDATE, 
			device_types.TypeCode,
			device_types.Name,
			device_types.Description,
			device_types.IsMobile,
			device_types.SecurityLevel,
			device_types.IsActive,
			device_types.CreatedAt,
			device_types.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Device_typesRepository.repository.UpdateDevice_types.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateDevice_types.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateDevice_types.rows_affected", rowsAffected)
	return nil
}

