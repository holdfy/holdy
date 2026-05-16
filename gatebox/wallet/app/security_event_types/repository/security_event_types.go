package security_event_typesRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Security_event_typesRepositoryIF interface {
     GetSecurity_event_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetSecurity_event_typesById(ctx context.Context, id int64) (*model.Security_event_types, error)
     GetSecurity_event_typesByTypeCode(ctx context.Context, typecode string) (*model.Security_event_types, error)
     InsertSecurity_event_types(ctx context.Context, security_event_types *model.Security_event_types) (int64, error)
     UpdateSecurity_event_types(ctx context.Context, security_event_types *model.Security_event_types, id int64) error
     DeleteSecurity_event_typesById(ctx context.Context, id int64) (bool, error)
}
 type Security_event_typesRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewSecurity_event_typesRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Security_event_typesRepository{
    return &Security_event_typesRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Security_event_types"),
     }
}
func (t Security_event_typesRepository)  GetSecurity_event_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_event_typesRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSecurity_event_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSecurity_event_types.offset", offset)
	tracker.AddParam("repository.GetSecurity_event_types.limit", limit)
	itemsPage 			= model.ItemsPage{}
	security_event_typess := []model.Security_event_types{}

	rows, err := t.PGRead.Query(ctx, SQL_SECURITY_EVENT_TYPES_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Security_event_typesRepository.repository.GetSecurity_event_typess.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var security_event_types model.Security_event_types
		err := rows.Scan(
			&security_event_types.ID,
			&security_event_types.TypeCode,
			&security_event_types.Name,
			&security_event_types.Description,
			&security_event_types.DefaultSeverity,
			&security_event_types.AutoBlock,
			&security_event_types.RequiresInvestigation,
			&security_event_types.IsActive,
			&security_event_types.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Security_event_typesRepository.repository.GetSecurity_event_typess.Scan: ", err.Error())
			return itemsPage, err
		}
		security_event_typess = append(security_event_typess, security_event_types)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Security_event_typesRepository.repository.GetSecurity_event_typess.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(security_event_typess) > 0 {
		qtyRecords = security_event_typess[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = security_event_typess

	tracker.AddResult("repository.GetSecurity_event_types.rows_returned", len(security_event_typess))
	tracker.AddResult("repository.GetSecurity_event_types.total_count", len(security_event_typess))

	return itemsPage, nil
}
func (t Security_event_typesRepository)  GetSecurity_event_typesById(ctx context.Context, id int64) (security_event_types *model.Security_event_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_event_typesRepository -> GetSecurity_event_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSecurity_event_typesById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSecurity_event_typesById.id", id)

	security_event_types = new(model.Security_event_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_SECURITY_EVENT_TYPES_BY_ID, id)
		err = row.Scan(
			&security_event_types.ID,
			&security_event_types.TypeCode,
			&security_event_types.Name,
			&security_event_types.Description,
			&security_event_types.DefaultSeverity,
			&security_event_types.AutoBlock,
			&security_event_types.RequiresInvestigation,
			&security_event_types.IsActive,
			&security_event_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Security_event_typesRepository.repository.GetSecurity_event_typesById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetSecurity_event_typesById.found", true)
	return security_event_types, nil
}
func (t Security_event_typesRepository)  GetSecurity_event_typesByTypeCode(ctx context.Context, typecode string) (security_event_types *model.Security_event_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_event_typesRepository -> GetSecurity_event_typesByTypeCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSecurity_event_typesByTypeCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSecurity_event_typesByTypeCode.typecode", typecode)

	security_event_types = new(model.Security_event_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_SECURITY_EVENT_TYPES_BY_TYPE_CODE, typecode)
		err = row.Scan(
			&security_event_types.ID,
			&security_event_types.TypeCode,
			&security_event_types.Name,
			&security_event_types.Description,
			&security_event_types.DefaultSeverity,
			&security_event_types.AutoBlock,
			&security_event_types.RequiresInvestigation,
			&security_event_types.IsActive,
			&security_event_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Security_event_typesRepository.repository.GetSecurity_event_typesBytypecode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return security_event_types, nil
}
func (t Security_event_typesRepository)  DeleteSecurity_event_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_event_typesRepository -> DeleteSecurity_event_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteSecurity_event_typesById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_SECURITY_EVENT_TYPES_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Security_event_typesRepository.repository.DeleteSecurity_event_typesById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteSecurity_event_typesById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteSecurity_event_typesById.deleted", result)
	return true, err
}
func (t Security_event_typesRepository)  InsertSecurity_event_types(ctx context.Context,security_event_types *model.Security_event_types) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_event_typesRepository -> InsertSecurity_event_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertSecurity_event_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertSecurity_event_types.typecode", security_event_types.TypeCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_SECURITY_EVENT_TYPES_INSERT,
			security_event_types.TypeCode,
			security_event_types.Name,
			security_event_types.Description,
			security_event_types.DefaultSeverity,
			security_event_types.AutoBlock,
			security_event_types.RequiresInvestigation,
			security_event_types.IsActive,
			security_event_types.CreatedAt,
	).Scan(&security_event_types.ID)

	if err != nil {
		t.log.Error(ctx, "Security_event_typesRepository.repository.InsertSecurity_event_types.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertSecurity_event_types.inserted_id", security_event_types.ID)
   return security_event_types.ID, nil

}
func (t Security_event_typesRepository)  UpdateSecurity_event_types(ctx context.Context,security_event_types *model.Security_event_types, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_event_typesRepository -> UpdateSecurity_event_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateSecurity_event_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateSecurity_event_types.id", id)
	tracker.AddParam("repository.UpdateSecurity_event_types.typecode", security_event_types.TypeCode)

	security_event_types.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_SECURITY_EVENT_TYPES_UPDATE, 
			security_event_types.TypeCode,
			security_event_types.Name,
			security_event_types.Description,
			security_event_types.DefaultSeverity,
			security_event_types.AutoBlock,
			security_event_types.RequiresInvestigation,
			security_event_types.IsActive,
			security_event_types.CreatedAt,
			security_event_types.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Security_event_typesRepository.repository.UpdateSecurity_event_types.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateSecurity_event_types.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateSecurity_event_types.rows_affected", rowsAffected)
	return nil
}

