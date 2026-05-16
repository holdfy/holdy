package notification_channelsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Notification_channelsRepositoryIF interface {
     GetNotification_channels(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetNotification_channelsById(ctx context.Context, id int64) (*model.Notification_channels, error)
     GetNotification_channelsByChannelCode(ctx context.Context, channelcode string) (*model.Notification_channels, error)
     InsertNotification_channels(ctx context.Context, notification_channels *model.Notification_channels) (int64, error)
     UpdateNotification_channels(ctx context.Context, notification_channels *model.Notification_channels, id int64) error
     DeleteNotification_channelsById(ctx context.Context, id int64) (bool, error)
}
 type Notification_channelsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewNotification_channelsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Notification_channelsRepository{
    return &Notification_channelsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Notification_channels"),
     }
}
func (t Notification_channelsRepository)  GetNotification_channels(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_channelsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetNotification_channels")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetNotification_channels.offset", offset)
	tracker.AddParam("repository.GetNotification_channels.limit", limit)
	itemsPage 			= model.ItemsPage{}
	notification_channelss := []model.Notification_channels{}

	rows, err := t.PGRead.Query(ctx, SQL_NOTIFICATION_CHANNELS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Notification_channelsRepository.repository.GetNotification_channelss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var notification_channels model.Notification_channels
		err := rows.Scan(
			&notification_channels.ID,
			&notification_channels.ChannelCode,
			&notification_channels.Name,
			&notification_channels.Description,
			&notification_channels.RequiresSubject,
			&notification_channels.MaxBodyLength,
			&notification_channels.DeliveryTimeSeconds,
			&notification_channels.IsActive,
			&notification_channels.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Notification_channelsRepository.repository.GetNotification_channelss.Scan: ", err.Error())
			return itemsPage, err
		}
		notification_channelss = append(notification_channelss, notification_channels)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Notification_channelsRepository.repository.GetNotification_channelss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(notification_channelss) > 0 {
		qtyRecords = notification_channelss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = notification_channelss

	tracker.AddResult("repository.GetNotification_channels.rows_returned", len(notification_channelss))
	tracker.AddResult("repository.GetNotification_channels.total_count", len(notification_channelss))

	return itemsPage, nil
}
func (t Notification_channelsRepository)  GetNotification_channelsById(ctx context.Context, id int64) (notification_channels *model.Notification_channels, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_channelsRepository -> GetNotification_channelsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetNotification_channelsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetNotification_channelsById.id", id)

	notification_channels = new(model.Notification_channels)
	row := t.PGRead.QueryRow(ctx, SQL_GET_NOTIFICATION_CHANNELS_BY_ID, id)
		err = row.Scan(
			&notification_channels.ID,
			&notification_channels.ChannelCode,
			&notification_channels.Name,
			&notification_channels.Description,
			&notification_channels.RequiresSubject,
			&notification_channels.MaxBodyLength,
			&notification_channels.DeliveryTimeSeconds,
			&notification_channels.IsActive,
			&notification_channels.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Notification_channelsRepository.repository.GetNotification_channelsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetNotification_channelsById.found", true)
	return notification_channels, nil
}
func (t Notification_channelsRepository)  GetNotification_channelsByChannelCode(ctx context.Context, channelcode string) (notification_channels *model.Notification_channels, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_channelsRepository -> GetNotification_channelsByChannelCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetNotification_channelsByChannelCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetNotification_channelsByChannelCode.channelcode", channelcode)

	notification_channels = new(model.Notification_channels)
	row := t.PGRead.QueryRow(ctx, SQL_GET_NOTIFICATION_CHANNELS_BY_CHANNEL_CODE, channelcode)
		err = row.Scan(
			&notification_channels.ID,
			&notification_channels.ChannelCode,
			&notification_channels.Name,
			&notification_channels.Description,
			&notification_channels.RequiresSubject,
			&notification_channels.MaxBodyLength,
			&notification_channels.DeliveryTimeSeconds,
			&notification_channels.IsActive,
			&notification_channels.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Notification_channelsRepository.repository.GetNotification_channelsBychannelcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return notification_channels, nil
}
func (t Notification_channelsRepository)  DeleteNotification_channelsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_channelsRepository -> DeleteNotification_channelsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteNotification_channelsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_NOTIFICATION_CHANNELS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Notification_channelsRepository.repository.DeleteNotification_channelsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteNotification_channelsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteNotification_channelsById.deleted", result)
	return true, err
}
func (t Notification_channelsRepository)  InsertNotification_channels(ctx context.Context,notification_channels *model.Notification_channels) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_channelsRepository -> InsertNotification_channels", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertNotification_channels")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertNotification_channels.channelcode", notification_channels.ChannelCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_NOTIFICATION_CHANNELS_INSERT,
			notification_channels.ChannelCode,
			notification_channels.Name,
			notification_channels.Description,
			notification_channels.RequiresSubject,
			notification_channels.MaxBodyLength,
			notification_channels.DeliveryTimeSeconds,
			notification_channels.IsActive,
			notification_channels.CreatedAt,
	).Scan(&notification_channels.ID)

	if err != nil {
		t.log.Error(ctx, "Notification_channelsRepository.repository.InsertNotification_channels.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertNotification_channels.inserted_id", notification_channels.ID)
   return notification_channels.ID, nil

}
func (t Notification_channelsRepository)  UpdateNotification_channels(ctx context.Context,notification_channels *model.Notification_channels, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_channelsRepository -> UpdateNotification_channels", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateNotification_channels")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateNotification_channels.id", id)
	tracker.AddParam("repository.UpdateNotification_channels.channelcode", notification_channels.ChannelCode)

	notification_channels.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_NOTIFICATION_CHANNELS_UPDATE, 
			notification_channels.ChannelCode,
			notification_channels.Name,
			notification_channels.Description,
			notification_channels.RequiresSubject,
			notification_channels.MaxBodyLength,
			notification_channels.DeliveryTimeSeconds,
			notification_channels.IsActive,
			notification_channels.CreatedAt,
			notification_channels.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Notification_channelsRepository.repository.UpdateNotification_channels.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateNotification_channels.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateNotification_channels.rows_affected", rowsAffected)
	return nil
}

