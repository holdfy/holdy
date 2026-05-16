package user_sessionsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type User_sessionsRepositoryIF interface {
     GetUser_sessions(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetUser_sessionsById(ctx context.Context, id int64) (*model.User_sessions, error)
     GetUser_sessionsBySessionCode(ctx context.Context, sessioncode string) (*model.User_sessions, error)
     InsertUser_sessions(ctx context.Context, user_sessions *model.User_sessions) (int64, error)
     UpdateUser_sessions(ctx context.Context, user_sessions *model.User_sessions, id int64) error
     DeleteUser_sessionsById(ctx context.Context, id int64) (bool, error)
}
 type User_sessionsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewUser_sessionsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *User_sessionsRepository{
    return &User_sessionsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("User_sessions"),
     }
}
func (t User_sessionsRepository)  GetUser_sessions(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_sessionsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_sessions")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_sessions.offset", offset)
	tracker.AddParam("repository.GetUser_sessions.limit", limit)
	itemsPage 			= model.ItemsPage{}
	user_sessionss := []model.User_sessions{}

	rows, err := t.PGRead.Query(ctx, SQL_USER_SESSIONS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "User_sessionsRepository.repository.GetUser_sessionss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var user_sessions model.User_sessions
		err := rows.Scan(
			&user_sessions.ID,
			&user_sessions.SessionCode,
			&user_sessions.UserId,
			&user_sessions.ApplicationId,
			&user_sessions.SessionToken,
			&user_sessions.DeviceFingerprint,
			&user_sessions.IdDeviceType,
			&user_sessions.DeviceInfo,
			&user_sessions.IpAddress,
			&user_sessions.LocationData,
			&user_sessions.IdStatus,
			&user_sessions.ExpiresAt,
			&user_sessions.LastActivity,
			&user_sessions.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "User_sessionsRepository.repository.GetUser_sessionss.Scan: ", err.Error())
			return itemsPage, err
		}
		user_sessionss = append(user_sessionss, user_sessions)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "User_sessionsRepository.repository.GetUser_sessionss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(user_sessionss) > 0 {
		qtyRecords = user_sessionss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = user_sessionss

	tracker.AddResult("repository.GetUser_sessions.rows_returned", len(user_sessionss))
	tracker.AddResult("repository.GetUser_sessions.total_count", len(user_sessionss))

	return itemsPage, nil
}
func (t User_sessionsRepository)  GetUser_sessionsById(ctx context.Context, id int64) (user_sessions *model.User_sessions, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_sessionsRepository -> GetUser_sessionsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_sessionsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_sessionsById.id", id)

	user_sessions = new(model.User_sessions)
	row := t.PGRead.QueryRow(ctx, SQL_GET_USER_SESSIONS_BY_ID, id)
		err = row.Scan(
			&user_sessions.ID,
			&user_sessions.SessionCode,
			&user_sessions.UserId,
			&user_sessions.ApplicationId,
			&user_sessions.SessionToken,
			&user_sessions.DeviceFingerprint,
			&user_sessions.IdDeviceType,
			&user_sessions.DeviceInfo,
			&user_sessions.IpAddress,
			&user_sessions.LocationData,
			&user_sessions.IdStatus,
			&user_sessions.ExpiresAt,
			&user_sessions.LastActivity,
			&user_sessions.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"User_sessionsRepository.repository.GetUser_sessionsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetUser_sessionsById.found", true)
	return user_sessions, nil
}
func (t User_sessionsRepository)  GetUser_sessionsBySessionCode(ctx context.Context, sessioncode string) (user_sessions *model.User_sessions, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_sessionsRepository -> GetUser_sessionsBySessionCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_sessionsBySessionCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_sessionsBySessionCode.sessioncode", sessioncode)

	user_sessions = new(model.User_sessions)
	row := t.PGRead.QueryRow(ctx, SQL_GET_USER_SESSIONS_BY_SESSION_CODE, sessioncode)
		err = row.Scan(
			&user_sessions.ID,
			&user_sessions.SessionCode,
			&user_sessions.UserId,
			&user_sessions.ApplicationId,
			&user_sessions.SessionToken,
			&user_sessions.DeviceFingerprint,
			&user_sessions.IdDeviceType,
			&user_sessions.DeviceInfo,
			&user_sessions.IpAddress,
			&user_sessions.LocationData,
			&user_sessions.IdStatus,
			&user_sessions.ExpiresAt,
			&user_sessions.LastActivity,
			&user_sessions.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"User_sessionsRepository.repository.GetUser_sessionsBysessioncode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return user_sessions, nil
}
func (t User_sessionsRepository)  DeleteUser_sessionsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_sessionsRepository -> DeleteUser_sessionsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteUser_sessionsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_USER_SESSIONS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"User_sessionsRepository.repository.DeleteUser_sessionsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteUser_sessionsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteUser_sessionsById.deleted", result)
	return true, err
}
func (t User_sessionsRepository)  InsertUser_sessions(ctx context.Context,user_sessions *model.User_sessions) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_sessionsRepository -> InsertUser_sessions", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertUser_sessions")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertUser_sessions.sessioncode", user_sessions.SessionCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_USER_SESSIONS_INSERT,
			user_sessions.SessionCode,
			user_sessions.UserId,
			user_sessions.ApplicationId,
			user_sessions.SessionToken,
			user_sessions.DeviceFingerprint,
			user_sessions.IdDeviceType,
			user_sessions.DeviceInfo,
			user_sessions.IpAddress,
			user_sessions.LocationData,
			user_sessions.IdStatus,
			user_sessions.ExpiresAt,
			user_sessions.LastActivity,
			user_sessions.CreatedAt,
	).Scan(&user_sessions.ID)

	if err != nil {
		t.log.Error(ctx, "User_sessionsRepository.repository.InsertUser_sessions.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertUser_sessions.inserted_id", user_sessions.ID)
   return user_sessions.ID, nil

}
func (t User_sessionsRepository)  UpdateUser_sessions(ctx context.Context,user_sessions *model.User_sessions, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_sessionsRepository -> UpdateUser_sessions", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateUser_sessions")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateUser_sessions.id", id)
	tracker.AddParam("repository.UpdateUser_sessions.sessioncode", user_sessions.SessionCode)

	user_sessions.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_USER_SESSIONS_UPDATE, 
			user_sessions.SessionCode,
			user_sessions.UserId,
			user_sessions.ApplicationId,
			user_sessions.SessionToken,
			user_sessions.DeviceFingerprint,
			user_sessions.IdDeviceType,
			user_sessions.DeviceInfo,
			user_sessions.IpAddress,
			user_sessions.LocationData,
			user_sessions.IdStatus,
			user_sessions.ExpiresAt,
			user_sessions.LastActivity,
			user_sessions.CreatedAt,
			user_sessions.ID,
   )
	if err != nil {
		t.log.Error(ctx, "User_sessionsRepository.repository.UpdateUser_sessions.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateUser_sessions.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateUser_sessions.rows_affected", rowsAffected)
	return nil
}

