package notification_statusRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Notification_statusRepositoryIF interface {
     GetNotification_status(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetNotification_statusById(ctx context.Context, id int64) (*model.Notification_status, error)
     GetNotification_statusByStatusCode(ctx context.Context, statuscode string) (*model.Notification_status, error)
     InsertNotification_status(ctx context.Context, notification_status *model.Notification_status) (int64, error)
     UpdateNotification_status(ctx context.Context, notification_status *model.Notification_status, id int64) error
     DeleteNotification_statusById(ctx context.Context, id int64) (bool, error)
}
 type Notification_statusRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewNotification_statusRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Notification_statusRepository{
    return &Notification_statusRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Notification_status"),
     }
}
func (t Notification_statusRepository)  GetNotification_status(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_statusRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetNotification_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetNotification_status.offset", offset)
	tracker.AddParam("repository.GetNotification_status.limit", limit)
	itemsPage 			= model.ItemsPage{}
	notification_statuss := []model.Notification_status{}

	rows, err := t.PGRead.Query(ctx, SQL_NOTIFICATION_STATUS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Notification_statusRepository.repository.GetNotification_statuss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var notification_status model.Notification_status
		err := rows.Scan(
			&notification_status.ID,
			&notification_status.StatusCode,
			&notification_status.Name,
			&notification_status.Description,
			&notification_status.IsFinal,
			&notification_status.IsSuccess,
			&notification_status.RequiresRetry,
			&notification_status.IsActive,
			&notification_status.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Notification_statusRepository.repository.GetNotification_statuss.Scan: ", err.Error())
			return itemsPage, err
		}
		notification_statuss = append(notification_statuss, notification_status)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Notification_statusRepository.repository.GetNotification_statuss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(notification_statuss) > 0 {
		qtyRecords = notification_statuss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = notification_statuss

	tracker.AddResult("repository.GetNotification_status.rows_returned", len(notification_statuss))
	tracker.AddResult("repository.GetNotification_status.total_count", len(notification_statuss))

	return itemsPage, nil
}
func (t Notification_statusRepository)  GetNotification_statusById(ctx context.Context, id int64) (notification_status *model.Notification_status, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_statusRepository -> GetNotification_statusById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetNotification_statusById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetNotification_statusById.id", id)

	notification_status = new(model.Notification_status)
	row := t.PGRead.QueryRow(ctx, SQL_GET_NOTIFICATION_STATUS_BY_ID, id)
		err = row.Scan(
			&notification_status.ID,
			&notification_status.StatusCode,
			&notification_status.Name,
			&notification_status.Description,
			&notification_status.IsFinal,
			&notification_status.IsSuccess,
			&notification_status.RequiresRetry,
			&notification_status.IsActive,
			&notification_status.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Notification_statusRepository.repository.GetNotification_statusById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetNotification_statusById.found", true)
	return notification_status, nil
}
func (t Notification_statusRepository)  GetNotification_statusByStatusCode(ctx context.Context, statuscode string) (notification_status *model.Notification_status, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_statusRepository -> GetNotification_statusByStatusCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetNotification_statusByStatusCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetNotification_statusByStatusCode.statuscode", statuscode)

	notification_status = new(model.Notification_status)
	row := t.PGRead.QueryRow(ctx, SQL_GET_NOTIFICATION_STATUS_BY_STATUS_CODE, statuscode)
		err = row.Scan(
			&notification_status.ID,
			&notification_status.StatusCode,
			&notification_status.Name,
			&notification_status.Description,
			&notification_status.IsFinal,
			&notification_status.IsSuccess,
			&notification_status.RequiresRetry,
			&notification_status.IsActive,
			&notification_status.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Notification_statusRepository.repository.GetNotification_statusBystatuscode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return notification_status, nil
}
func (t Notification_statusRepository)  DeleteNotification_statusById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_statusRepository -> DeleteNotification_statusById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteNotification_statusById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_NOTIFICATION_STATUS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Notification_statusRepository.repository.DeleteNotification_statusById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteNotification_statusById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteNotification_statusById.deleted", result)
	return true, err
}
func (t Notification_statusRepository)  InsertNotification_status(ctx context.Context,notification_status *model.Notification_status) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_statusRepository -> InsertNotification_status", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertNotification_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertNotification_status.statuscode", notification_status.StatusCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_NOTIFICATION_STATUS_INSERT,
			notification_status.StatusCode,
			notification_status.Name,
			notification_status.Description,
			notification_status.IsFinal,
			notification_status.IsSuccess,
			notification_status.RequiresRetry,
			notification_status.IsActive,
			notification_status.CreatedAt,
	).Scan(&notification_status.ID)

	if err != nil {
		t.log.Error(ctx, "Notification_statusRepository.repository.InsertNotification_status.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertNotification_status.inserted_id", notification_status.ID)
   return notification_status.ID, nil

}
func (t Notification_statusRepository)  UpdateNotification_status(ctx context.Context,notification_status *model.Notification_status, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_statusRepository -> UpdateNotification_status", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateNotification_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateNotification_status.id", id)
	tracker.AddParam("repository.UpdateNotification_status.statuscode", notification_status.StatusCode)

	notification_status.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_NOTIFICATION_STATUS_UPDATE, 
			notification_status.StatusCode,
			notification_status.Name,
			notification_status.Description,
			notification_status.IsFinal,
			notification_status.IsSuccess,
			notification_status.RequiresRetry,
			notification_status.IsActive,
			notification_status.CreatedAt,
			notification_status.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Notification_statusRepository.repository.UpdateNotification_status.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateNotification_status.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateNotification_status.rows_affected", rowsAffected)
	return nil
}

