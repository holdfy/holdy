package notification_historyRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Notification_historyRepositoryIF interface {
     GetNotification_history(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetNotification_historyById(ctx context.Context, id int64) (*model.Notification_history, error)
     GetNotification_historyByNotificationCode(ctx context.Context, notificationcode string) (*model.Notification_history, error)
     InsertNotification_history(ctx context.Context, notification_history *model.Notification_history) (int64, error)
     UpdateNotification_history(ctx context.Context, notification_history *model.Notification_history, id int64) error
     DeleteNotification_historyById(ctx context.Context, id int64) (bool, error)
}
 type Notification_historyRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewNotification_historyRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Notification_historyRepository{
    return &Notification_historyRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Notification_history"),
     }
}
func (t Notification_historyRepository)  GetNotification_history(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_historyRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetNotification_history")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetNotification_history.offset", offset)
	tracker.AddParam("repository.GetNotification_history.limit", limit)
	itemsPage 			= model.ItemsPage{}
	notification_historys := []model.Notification_history{}

	rows, err := t.PGRead.Query(ctx, SQL_NOTIFICATION_HISTORY_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Notification_historyRepository.repository.GetNotification_historys.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var notification_history model.Notification_history
		err := rows.Scan(
			&notification_history.ID,
			&notification_history.NotificationCode,
			&notification_history.UserId,
			&notification_history.TransactionId,
			&notification_history.TemplateId,
			&notification_history.IdChannel,
			&notification_history.Recipient,
			&notification_history.Subject,
			&notification_history.MessageBody,
			&notification_history.IdStatus,
			&notification_history.ProviderResponse,
			&notification_history.SentAt,
			&notification_history.DeliveredAt,
			&notification_history.FailedAt,
			&notification_history.FailureReason,
			&notification_history.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Notification_historyRepository.repository.GetNotification_historys.Scan: ", err.Error())
			return itemsPage, err
		}
		notification_historys = append(notification_historys, notification_history)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Notification_historyRepository.repository.GetNotification_historys.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(notification_historys) > 0 {
		qtyRecords = notification_historys[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = notification_historys

	tracker.AddResult("repository.GetNotification_history.rows_returned", len(notification_historys))
	tracker.AddResult("repository.GetNotification_history.total_count", len(notification_historys))

	return itemsPage, nil
}
func (t Notification_historyRepository)  GetNotification_historyById(ctx context.Context, id int64) (notification_history *model.Notification_history, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_historyRepository -> GetNotification_historyById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetNotification_historyById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetNotification_historyById.id", id)

	notification_history = new(model.Notification_history)
	row := t.PGRead.QueryRow(ctx, SQL_GET_NOTIFICATION_HISTORY_BY_ID, id)
		err = row.Scan(
			&notification_history.ID,
			&notification_history.NotificationCode,
			&notification_history.UserId,
			&notification_history.TransactionId,
			&notification_history.TemplateId,
			&notification_history.IdChannel,
			&notification_history.Recipient,
			&notification_history.Subject,
			&notification_history.MessageBody,
			&notification_history.IdStatus,
			&notification_history.ProviderResponse,
			&notification_history.SentAt,
			&notification_history.DeliveredAt,
			&notification_history.FailedAt,
			&notification_history.FailureReason,
			&notification_history.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Notification_historyRepository.repository.GetNotification_historyById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetNotification_historyById.found", true)
	return notification_history, nil
}
func (t Notification_historyRepository)  GetNotification_historyByNotificationCode(ctx context.Context, notificationcode string) (notification_history *model.Notification_history, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_historyRepository -> GetNotification_historyByNotificationCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetNotification_historyByNotificationCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetNotification_historyByNotificationCode.notificationcode", notificationcode)

	notification_history = new(model.Notification_history)
	row := t.PGRead.QueryRow(ctx, SQL_GET_NOTIFICATION_HISTORY_BY_NOTIFICATION_CODE, notificationcode)
		err = row.Scan(
			&notification_history.ID,
			&notification_history.NotificationCode,
			&notification_history.UserId,
			&notification_history.TransactionId,
			&notification_history.TemplateId,
			&notification_history.IdChannel,
			&notification_history.Recipient,
			&notification_history.Subject,
			&notification_history.MessageBody,
			&notification_history.IdStatus,
			&notification_history.ProviderResponse,
			&notification_history.SentAt,
			&notification_history.DeliveredAt,
			&notification_history.FailedAt,
			&notification_history.FailureReason,
			&notification_history.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Notification_historyRepository.repository.GetNotification_historyBynotificationcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return notification_history, nil
}
func (t Notification_historyRepository)  DeleteNotification_historyById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_historyRepository -> DeleteNotification_historyById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteNotification_historyById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_NOTIFICATION_HISTORY_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Notification_historyRepository.repository.DeleteNotification_historyById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteNotification_historyById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteNotification_historyById.deleted", result)
	return true, err
}
func (t Notification_historyRepository)  InsertNotification_history(ctx context.Context,notification_history *model.Notification_history) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_historyRepository -> InsertNotification_history", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertNotification_history")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertNotification_history.notificationcode", notification_history.NotificationCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_NOTIFICATION_HISTORY_INSERT,
			notification_history.NotificationCode,
			notification_history.UserId,
			notification_history.TransactionId,
			notification_history.TemplateId,
			notification_history.IdChannel,
			notification_history.Recipient,
			notification_history.Subject,
			notification_history.MessageBody,
			notification_history.IdStatus,
			notification_history.ProviderResponse,
			notification_history.SentAt,
			notification_history.DeliveredAt,
			notification_history.FailedAt,
			notification_history.FailureReason,
			notification_history.CreatedAt,
	).Scan(&notification_history.ID)

	if err != nil {
		t.log.Error(ctx, "Notification_historyRepository.repository.InsertNotification_history.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertNotification_history.inserted_id", notification_history.ID)
   return notification_history.ID, nil

}
func (t Notification_historyRepository)  UpdateNotification_history(ctx context.Context,notification_history *model.Notification_history, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_historyRepository -> UpdateNotification_history", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateNotification_history")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateNotification_history.id", id)
	tracker.AddParam("repository.UpdateNotification_history.notificationcode", notification_history.NotificationCode)

	notification_history.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_NOTIFICATION_HISTORY_UPDATE, 
			notification_history.NotificationCode,
			notification_history.UserId,
			notification_history.TransactionId,
			notification_history.TemplateId,
			notification_history.IdChannel,
			notification_history.Recipient,
			notification_history.Subject,
			notification_history.MessageBody,
			notification_history.IdStatus,
			notification_history.ProviderResponse,
			notification_history.SentAt,
			notification_history.DeliveredAt,
			notification_history.FailedAt,
			notification_history.FailureReason,
			notification_history.CreatedAt,
			notification_history.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Notification_historyRepository.repository.UpdateNotification_history.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateNotification_history.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateNotification_history.rows_affected", rowsAffected)
	return nil
}

