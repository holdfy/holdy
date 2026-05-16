package session_statusRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Session_statusRepositoryIF interface {
     GetSession_status(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetSession_statusById(ctx context.Context, id int64) (*model.Session_status, error)
     GetSession_statusByStatusCode(ctx context.Context, statuscode string) (*model.Session_status, error)
     InsertSession_status(ctx context.Context, session_status *model.Session_status) (int64, error)
     UpdateSession_status(ctx context.Context, session_status *model.Session_status, id int64) error
     DeleteSession_statusById(ctx context.Context, id int64) (bool, error)
}
 type Session_statusRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewSession_statusRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Session_statusRepository{
    return &Session_statusRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Session_status"),
     }
}
func (t Session_statusRepository)  GetSession_status(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Session_statusRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSession_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSession_status.offset", offset)
	tracker.AddParam("repository.GetSession_status.limit", limit)
	itemsPage 			= model.ItemsPage{}
	session_statuss := []model.Session_status{}

	rows, err := t.PGRead.Query(ctx, SQL_SESSION_STATUS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Session_statusRepository.repository.GetSession_statuss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var session_status model.Session_status
		err := rows.Scan(
			&session_status.ID,
			&session_status.StatusCode,
			&session_status.Name,
			&session_status.Description,
			&session_status.AllowsActivity,
			&session_status.IsActive,
			&session_status.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Session_statusRepository.repository.GetSession_statuss.Scan: ", err.Error())
			return itemsPage, err
		}
		session_statuss = append(session_statuss, session_status)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Session_statusRepository.repository.GetSession_statuss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(session_statuss) > 0 {
		qtyRecords = session_statuss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = session_statuss

	tracker.AddResult("repository.GetSession_status.rows_returned", len(session_statuss))
	tracker.AddResult("repository.GetSession_status.total_count", len(session_statuss))

	return itemsPage, nil
}
func (t Session_statusRepository)  GetSession_statusById(ctx context.Context, id int64) (session_status *model.Session_status, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Session_statusRepository -> GetSession_statusById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSession_statusById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSession_statusById.id", id)

	session_status = new(model.Session_status)
	row := t.PGRead.QueryRow(ctx, SQL_GET_SESSION_STATUS_BY_ID, id)
		err = row.Scan(
			&session_status.ID,
			&session_status.StatusCode,
			&session_status.Name,
			&session_status.Description,
			&session_status.AllowsActivity,
			&session_status.IsActive,
			&session_status.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Session_statusRepository.repository.GetSession_statusById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetSession_statusById.found", true)
	return session_status, nil
}
func (t Session_statusRepository)  GetSession_statusByStatusCode(ctx context.Context, statuscode string) (session_status *model.Session_status, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Session_statusRepository -> GetSession_statusByStatusCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSession_statusByStatusCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSession_statusByStatusCode.statuscode", statuscode)

	session_status = new(model.Session_status)
	row := t.PGRead.QueryRow(ctx, SQL_GET_SESSION_STATUS_BY_STATUS_CODE, statuscode)
		err = row.Scan(
			&session_status.ID,
			&session_status.StatusCode,
			&session_status.Name,
			&session_status.Description,
			&session_status.AllowsActivity,
			&session_status.IsActive,
			&session_status.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Session_statusRepository.repository.GetSession_statusBystatuscode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return session_status, nil
}
func (t Session_statusRepository)  DeleteSession_statusById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Session_statusRepository -> DeleteSession_statusById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteSession_statusById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_SESSION_STATUS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Session_statusRepository.repository.DeleteSession_statusById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteSession_statusById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteSession_statusById.deleted", result)
	return true, err
}
func (t Session_statusRepository)  InsertSession_status(ctx context.Context,session_status *model.Session_status) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Session_statusRepository -> InsertSession_status", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertSession_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertSession_status.statuscode", session_status.StatusCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_SESSION_STATUS_INSERT,
			session_status.StatusCode,
			session_status.Name,
			session_status.Description,
			session_status.AllowsActivity,
			session_status.IsActive,
			session_status.CreatedAt,
	).Scan(&session_status.ID)

	if err != nil {
		t.log.Error(ctx, "Session_statusRepository.repository.InsertSession_status.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertSession_status.inserted_id", session_status.ID)
   return session_status.ID, nil

}
func (t Session_statusRepository)  UpdateSession_status(ctx context.Context,session_status *model.Session_status, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Session_statusRepository -> UpdateSession_status", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateSession_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateSession_status.id", id)
	tracker.AddParam("repository.UpdateSession_status.statuscode", session_status.StatusCode)

	session_status.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_SESSION_STATUS_UPDATE, 
			session_status.StatusCode,
			session_status.Name,
			session_status.Description,
			session_status.AllowsActivity,
			session_status.IsActive,
			session_status.CreatedAt,
			session_status.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Session_statusRepository.repository.UpdateSession_status.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateSession_status.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateSession_status.rows_affected", rowsAffected)
	return nil
}

