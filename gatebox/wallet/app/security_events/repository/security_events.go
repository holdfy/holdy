package security_eventsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Security_eventsRepositoryIF interface {
     GetSecurity_events(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetSecurity_eventsById(ctx context.Context, id int64) (*model.Security_events, error)
     GetSecurity_eventsBySecurityEventCode(ctx context.Context, securityeventcode string) (*model.Security_events, error)
     InsertSecurity_events(ctx context.Context, security_events *model.Security_events) (int64, error)
     UpdateSecurity_events(ctx context.Context, security_events *model.Security_events, id int64) error
     DeleteSecurity_eventsById(ctx context.Context, id int64) (bool, error)
}
 type Security_eventsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewSecurity_eventsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Security_eventsRepository{
    return &Security_eventsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Security_events"),
     }
}
func (t Security_eventsRepository)  GetSecurity_events(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_eventsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSecurity_events")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSecurity_events.offset", offset)
	tracker.AddParam("repository.GetSecurity_events.limit", limit)
	itemsPage 			= model.ItemsPage{}
	security_eventss := []model.Security_events{}

	rows, err := t.PGRead.Query(ctx, SQL_SECURITY_EVENTS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Security_eventsRepository.repository.GetSecurity_eventss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var security_events model.Security_events
		err := rows.Scan(
			&security_events.ID,
			&security_events.SecurityEventCode,
			&security_events.UserId,
			&security_events.IdEventType,
			&security_events.IdSeverity,
			&security_events.Description,
			&security_events.SourceIp,
			&security_events.DeviceInfo,
			&security_events.Metadata,
			&security_events.IsResolved,
			&security_events.ResolvedAt,
			&security_events.ResolvedBy,
			&security_events.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Security_eventsRepository.repository.GetSecurity_eventss.Scan: ", err.Error())
			return itemsPage, err
		}
		security_eventss = append(security_eventss, security_events)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Security_eventsRepository.repository.GetSecurity_eventss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(security_eventss) > 0 {
		qtyRecords = security_eventss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = security_eventss

	tracker.AddResult("repository.GetSecurity_events.rows_returned", len(security_eventss))
	tracker.AddResult("repository.GetSecurity_events.total_count", len(security_eventss))

	return itemsPage, nil
}
func (t Security_eventsRepository)  GetSecurity_eventsById(ctx context.Context, id int64) (security_events *model.Security_events, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_eventsRepository -> GetSecurity_eventsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSecurity_eventsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSecurity_eventsById.id", id)

	security_events = new(model.Security_events)
	row := t.PGRead.QueryRow(ctx, SQL_GET_SECURITY_EVENTS_BY_ID, id)
		err = row.Scan(
			&security_events.ID,
			&security_events.SecurityEventCode,
			&security_events.UserId,
			&security_events.IdEventType,
			&security_events.IdSeverity,
			&security_events.Description,
			&security_events.SourceIp,
			&security_events.DeviceInfo,
			&security_events.Metadata,
			&security_events.IsResolved,
			&security_events.ResolvedAt,
			&security_events.ResolvedBy,
			&security_events.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Security_eventsRepository.repository.GetSecurity_eventsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetSecurity_eventsById.found", true)
	return security_events, nil
}
func (t Security_eventsRepository)  GetSecurity_eventsBySecurityEventCode(ctx context.Context, securityeventcode string) (security_events *model.Security_events, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_eventsRepository -> GetSecurity_eventsBySecurityEventCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSecurity_eventsBySecurityEventCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSecurity_eventsBySecurityEventCode.securityeventcode", securityeventcode)

	security_events = new(model.Security_events)
	row := t.PGRead.QueryRow(ctx, SQL_GET_SECURITY_EVENTS_BY_SECURITY_EVENT_CODE, securityeventcode)
		err = row.Scan(
			&security_events.ID,
			&security_events.SecurityEventCode,
			&security_events.UserId,
			&security_events.IdEventType,
			&security_events.IdSeverity,
			&security_events.Description,
			&security_events.SourceIp,
			&security_events.DeviceInfo,
			&security_events.Metadata,
			&security_events.IsResolved,
			&security_events.ResolvedAt,
			&security_events.ResolvedBy,
			&security_events.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Security_eventsRepository.repository.GetSecurity_eventsBysecurityeventcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return security_events, nil
}
func (t Security_eventsRepository)  DeleteSecurity_eventsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_eventsRepository -> DeleteSecurity_eventsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteSecurity_eventsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_SECURITY_EVENTS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Security_eventsRepository.repository.DeleteSecurity_eventsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteSecurity_eventsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteSecurity_eventsById.deleted", result)
	return true, err
}
func (t Security_eventsRepository)  InsertSecurity_events(ctx context.Context,security_events *model.Security_events) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_eventsRepository -> InsertSecurity_events", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertSecurity_events")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertSecurity_events.securityeventcode", security_events.SecurityEventCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_SECURITY_EVENTS_INSERT,
			security_events.SecurityEventCode,
			security_events.UserId,
			security_events.IdEventType,
			security_events.IdSeverity,
			security_events.Description,
			security_events.SourceIp,
			security_events.DeviceInfo,
			security_events.Metadata,
			security_events.IsResolved,
			security_events.ResolvedAt,
			security_events.ResolvedBy,
			security_events.CreatedAt,
	).Scan(&security_events.ID)

	if err != nil {
		t.log.Error(ctx, "Security_eventsRepository.repository.InsertSecurity_events.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertSecurity_events.inserted_id", security_events.ID)
   return security_events.ID, nil

}
func (t Security_eventsRepository)  UpdateSecurity_events(ctx context.Context,security_events *model.Security_events, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_eventsRepository -> UpdateSecurity_events", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateSecurity_events")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateSecurity_events.id", id)
	tracker.AddParam("repository.UpdateSecurity_events.securityeventcode", security_events.SecurityEventCode)

	security_events.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_SECURITY_EVENTS_UPDATE, 
			security_events.SecurityEventCode,
			security_events.UserId,
			security_events.IdEventType,
			security_events.IdSeverity,
			security_events.Description,
			security_events.SourceIp,
			security_events.DeviceInfo,
			security_events.Metadata,
			security_events.IsResolved,
			security_events.ResolvedAt,
			security_events.ResolvedBy,
			security_events.CreatedAt,
			security_events.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Security_eventsRepository.repository.UpdateSecurity_events.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateSecurity_events.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateSecurity_events.rows_affected", rowsAffected)
	return nil
}

