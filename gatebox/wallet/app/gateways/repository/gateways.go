package gatewaysRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type GatewaysRepositoryIF interface {
     GetGateways(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetGatewaysById(ctx context.Context, id int64) (*model.Gateways, error)
     GetGatewaysByGatewayCode(ctx context.Context, gatewaycode string) (*model.Gateways, error)
     InsertGateways(ctx context.Context, gateways *model.Gateways) (int64, error)
     UpdateGateways(ctx context.Context, gateways *model.Gateways, id int64) error
     DeleteGatewaysById(ctx context.Context, id int64) (bool, error)
}
 type GatewaysRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewGatewaysRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *GatewaysRepository{
    return &GatewaysRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Gateways"),
     }
}
func (t GatewaysRepository)  GetGateways(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("GatewaysRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetGateways")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetGateways.offset", offset)
	tracker.AddParam("repository.GetGateways.limit", limit)
	itemsPage 			= model.ItemsPage{}
	gatewayss := []model.Gateways{}

	rows, err := t.PGRead.Query(ctx, SQL_GATEWAYS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "GatewaysRepository.repository.GetGatewayss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var gateways model.Gateways
		err := rows.Scan(
			&gateways.ID,
			&gateways.GatewayCode,
			&gateways.Name,
			&gateways.Description,
			&gateways.ApiEndpoint,
			&gateways.TimeoutSeconds,
			&gateways.MaxRetries,
			&gateways.IsActive,
			&gateways.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "GatewaysRepository.repository.GetGatewayss.Scan: ", err.Error())
			return itemsPage, err
		}
		gatewayss = append(gatewayss, gateways)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "GatewaysRepository.repository.GetGatewayss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(gatewayss) > 0 {
		qtyRecords = gatewayss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = gatewayss

	tracker.AddResult("repository.GetGateways.rows_returned", len(gatewayss))
	tracker.AddResult("repository.GetGateways.total_count", len(gatewayss))

	return itemsPage, nil
}
func (t GatewaysRepository)  GetGatewaysById(ctx context.Context, id int64) (gateways *model.Gateways, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("GatewaysRepository -> GetGatewaysById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetGatewaysById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetGatewaysById.id", id)

	gateways = new(model.Gateways)
	row := t.PGRead.QueryRow(ctx, SQL_GET_GATEWAYS_BY_ID, id)
		err = row.Scan(
			&gateways.ID,
			&gateways.GatewayCode,
			&gateways.Name,
			&gateways.Description,
			&gateways.ApiEndpoint,
			&gateways.TimeoutSeconds,
			&gateways.MaxRetries,
			&gateways.IsActive,
			&gateways.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"GatewaysRepository.repository.GetGatewaysById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetGatewaysById.found", true)
	return gateways, nil
}
func (t GatewaysRepository)  GetGatewaysByGatewayCode(ctx context.Context, gatewaycode string) (gateways *model.Gateways, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("GatewaysRepository -> GetGatewaysByGatewayCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetGatewaysByGatewayCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetGatewaysByGatewayCode.gatewaycode", gatewaycode)

	gateways = new(model.Gateways)
	row := t.PGRead.QueryRow(ctx, SQL_GET_GATEWAYS_BY_GATEWAY_CODE, gatewaycode)
		err = row.Scan(
			&gateways.ID,
			&gateways.GatewayCode,
			&gateways.Name,
			&gateways.Description,
			&gateways.ApiEndpoint,
			&gateways.TimeoutSeconds,
			&gateways.MaxRetries,
			&gateways.IsActive,
			&gateways.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"GatewaysRepository.repository.GetGatewaysBygatewaycode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return gateways, nil
}
func (t GatewaysRepository)  DeleteGatewaysById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("GatewaysRepository -> DeleteGatewaysById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteGatewaysById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_GATEWAYS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"GatewaysRepository.repository.DeleteGatewaysById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteGatewaysById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteGatewaysById.deleted", result)
	return true, err
}
func (t GatewaysRepository)  InsertGateways(ctx context.Context,gateways *model.Gateways) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("GatewaysRepository -> InsertGateways", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertGateways")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertGateways.gatewaycode", gateways.GatewayCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_GATEWAYS_INSERT,
			gateways.GatewayCode,
			gateways.Name,
			gateways.Description,
			gateways.ApiEndpoint,
			gateways.TimeoutSeconds,
			gateways.MaxRetries,
			gateways.IsActive,
			gateways.CreatedAt,
	).Scan(&gateways.ID)

	if err != nil {
		t.log.Error(ctx, "GatewaysRepository.repository.InsertGateways.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertGateways.inserted_id", gateways.ID)
   return gateways.ID, nil

}
func (t GatewaysRepository)  UpdateGateways(ctx context.Context,gateways *model.Gateways, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("GatewaysRepository -> UpdateGateways", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateGateways")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateGateways.id", id)
	tracker.AddParam("repository.UpdateGateways.gatewaycode", gateways.GatewayCode)

	gateways.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_GATEWAYS_UPDATE, 
			gateways.GatewayCode,
			gateways.Name,
			gateways.Description,
			gateways.ApiEndpoint,
			gateways.TimeoutSeconds,
			gateways.MaxRetries,
			gateways.IsActive,
			gateways.CreatedAt,
			gateways.ID,
   )
	if err != nil {
		t.log.Error(ctx, "GatewaysRepository.repository.UpdateGateways.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateGateways.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateGateways.rows_affected", rowsAffected)
	return nil
}

