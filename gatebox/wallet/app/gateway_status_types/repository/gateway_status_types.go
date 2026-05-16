package gateway_status_typesRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Gateway_status_typesRepositoryIF interface {
     GetGateway_status_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetGateway_status_typesById(ctx context.Context, id int64) (*model.Gateway_status_types, error)
     GetGateway_status_typesByStatusCode(ctx context.Context, statuscode string) (*model.Gateway_status_types, error)
     InsertGateway_status_types(ctx context.Context, gateway_status_types *model.Gateway_status_types) (int64, error)
     UpdateGateway_status_types(ctx context.Context, gateway_status_types *model.Gateway_status_types, id int64) error
     DeleteGateway_status_typesById(ctx context.Context, id int64) (bool, error)
}
 type Gateway_status_typesRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewGateway_status_typesRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Gateway_status_typesRepository{
    return &Gateway_status_typesRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Gateway_status_types"),
     }
}
func (t Gateway_status_typesRepository)  GetGateway_status_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Gateway_status_typesRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetGateway_status_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetGateway_status_types.offset", offset)
	tracker.AddParam("repository.GetGateway_status_types.limit", limit)
	itemsPage 			= model.ItemsPage{}
	gateway_status_typess := []model.Gateway_status_types{}

	rows, err := t.PGRead.Query(ctx, SQL_GATEWAY_STATUS_TYPES_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Gateway_status_typesRepository.repository.GetGateway_status_typess.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var gateway_status_types model.Gateway_status_types
		err := rows.Scan(
			&gateway_status_types.ID,
			&gateway_status_types.StatusCode,
			&gateway_status_types.Name,
			&gateway_status_types.Description,
			&gateway_status_types.IsSuccess,
			&gateway_status_types.IsFinal,
			&gateway_status_types.IsActive,
			&gateway_status_types.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Gateway_status_typesRepository.repository.GetGateway_status_typess.Scan: ", err.Error())
			return itemsPage, err
		}
		gateway_status_typess = append(gateway_status_typess, gateway_status_types)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Gateway_status_typesRepository.repository.GetGateway_status_typess.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(gateway_status_typess) > 0 {
		qtyRecords = gateway_status_typess[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = gateway_status_typess

	tracker.AddResult("repository.GetGateway_status_types.rows_returned", len(gateway_status_typess))
	tracker.AddResult("repository.GetGateway_status_types.total_count", len(gateway_status_typess))

	return itemsPage, nil
}
func (t Gateway_status_typesRepository)  GetGateway_status_typesById(ctx context.Context, id int64) (gateway_status_types *model.Gateway_status_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Gateway_status_typesRepository -> GetGateway_status_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetGateway_status_typesById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetGateway_status_typesById.id", id)

	gateway_status_types = new(model.Gateway_status_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_GATEWAY_STATUS_TYPES_BY_ID, id)
		err = row.Scan(
			&gateway_status_types.ID,
			&gateway_status_types.StatusCode,
			&gateway_status_types.Name,
			&gateway_status_types.Description,
			&gateway_status_types.IsSuccess,
			&gateway_status_types.IsFinal,
			&gateway_status_types.IsActive,
			&gateway_status_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Gateway_status_typesRepository.repository.GetGateway_status_typesById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetGateway_status_typesById.found", true)
	return gateway_status_types, nil
}
func (t Gateway_status_typesRepository)  GetGateway_status_typesByStatusCode(ctx context.Context, statuscode string) (gateway_status_types *model.Gateway_status_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Gateway_status_typesRepository -> GetGateway_status_typesByStatusCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetGateway_status_typesByStatusCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetGateway_status_typesByStatusCode.statuscode", statuscode)

	gateway_status_types = new(model.Gateway_status_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_GATEWAY_STATUS_TYPES_BY_STATUS_CODE, statuscode)
		err = row.Scan(
			&gateway_status_types.ID,
			&gateway_status_types.StatusCode,
			&gateway_status_types.Name,
			&gateway_status_types.Description,
			&gateway_status_types.IsSuccess,
			&gateway_status_types.IsFinal,
			&gateway_status_types.IsActive,
			&gateway_status_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Gateway_status_typesRepository.repository.GetGateway_status_typesBystatuscode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return gateway_status_types, nil
}
func (t Gateway_status_typesRepository)  DeleteGateway_status_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Gateway_status_typesRepository -> DeleteGateway_status_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteGateway_status_typesById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_GATEWAY_STATUS_TYPES_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Gateway_status_typesRepository.repository.DeleteGateway_status_typesById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteGateway_status_typesById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteGateway_status_typesById.deleted", result)
	return true, err
}
func (t Gateway_status_typesRepository)  InsertGateway_status_types(ctx context.Context,gateway_status_types *model.Gateway_status_types) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Gateway_status_typesRepository -> InsertGateway_status_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertGateway_status_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertGateway_status_types.statuscode", gateway_status_types.StatusCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_GATEWAY_STATUS_TYPES_INSERT,
			gateway_status_types.StatusCode,
			gateway_status_types.Name,
			gateway_status_types.Description,
			gateway_status_types.IsSuccess,
			gateway_status_types.IsFinal,
			gateway_status_types.IsActive,
			gateway_status_types.CreatedAt,
	).Scan(&gateway_status_types.ID)

	if err != nil {
		t.log.Error(ctx, "Gateway_status_typesRepository.repository.InsertGateway_status_types.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertGateway_status_types.inserted_id", gateway_status_types.ID)
   return gateway_status_types.ID, nil

}
func (t Gateway_status_typesRepository)  UpdateGateway_status_types(ctx context.Context,gateway_status_types *model.Gateway_status_types, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Gateway_status_typesRepository -> UpdateGateway_status_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateGateway_status_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateGateway_status_types.id", id)
	tracker.AddParam("repository.UpdateGateway_status_types.statuscode", gateway_status_types.StatusCode)

	gateway_status_types.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_GATEWAY_STATUS_TYPES_UPDATE, 
			gateway_status_types.StatusCode,
			gateway_status_types.Name,
			gateway_status_types.Description,
			gateway_status_types.IsSuccess,
			gateway_status_types.IsFinal,
			gateway_status_types.IsActive,
			gateway_status_types.CreatedAt,
			gateway_status_types.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Gateway_status_typesRepository.repository.UpdateGateway_status_types.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateGateway_status_types.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateGateway_status_types.rows_affected", rowsAffected)
	return nil
}

