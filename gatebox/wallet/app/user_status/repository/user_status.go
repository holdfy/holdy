package user_statusRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type User_statusRepositoryIF interface {
     GetUser_status(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetUser_statusById(ctx context.Context, id int64) (*model.User_status, error)
     GetUser_statusByStatusCode(ctx context.Context, statuscode string) (*model.User_status, error)
     InsertUser_status(ctx context.Context, user_status *model.User_status) (int64, error)
     UpdateUser_status(ctx context.Context, user_status *model.User_status, id int64) error
     DeleteUser_statusById(ctx context.Context, id int64) (bool, error)
}
 type User_statusRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewUser_statusRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *User_statusRepository{
    return &User_statusRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("User_status"),
     }
}
func (t User_statusRepository)  GetUser_status(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_statusRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_status.offset", offset)
	tracker.AddParam("repository.GetUser_status.limit", limit)
	itemsPage 			= model.ItemsPage{}
	user_statuss := []model.User_status{}

	rows, err := t.PGRead.Query(ctx, SQL_USER_STATUS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "User_statusRepository.repository.GetUser_statuss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var user_status model.User_status
		err := rows.Scan(
			&user_status.ID,
			&user_status.StatusCode,
			&user_status.Name,
			&user_status.Description,
			&user_status.IsActive,
			&user_status.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "User_statusRepository.repository.GetUser_statuss.Scan: ", err.Error())
			return itemsPage, err
		}
		user_statuss = append(user_statuss, user_status)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "User_statusRepository.repository.GetUser_statuss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(user_statuss) > 0 {
		qtyRecords = user_statuss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = user_statuss

	tracker.AddResult("repository.GetUser_status.rows_returned", len(user_statuss))
	tracker.AddResult("repository.GetUser_status.total_count", len(user_statuss))

	return itemsPage, nil
}
func (t User_statusRepository)  GetUser_statusById(ctx context.Context, id int64) (user_status *model.User_status, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_statusRepository -> GetUser_statusById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_statusById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_statusById.id", id)

	user_status = new(model.User_status)
	row := t.PGRead.QueryRow(ctx, SQL_GET_USER_STATUS_BY_ID, id)
		err = row.Scan(
			&user_status.ID,
			&user_status.StatusCode,
			&user_status.Name,
			&user_status.Description,
			&user_status.IsActive,
			&user_status.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"User_statusRepository.repository.GetUser_statusById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetUser_statusById.found", true)
	return user_status, nil
}
func (t User_statusRepository)  GetUser_statusByStatusCode(ctx context.Context, statuscode string) (user_status *model.User_status, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_statusRepository -> GetUser_statusByStatusCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_statusByStatusCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_statusByStatusCode.statuscode", statuscode)

	user_status = new(model.User_status)
	row := t.PGRead.QueryRow(ctx, SQL_GET_USER_STATUS_BY_STATUS_CODE, statuscode)
		err = row.Scan(
			&user_status.ID,
			&user_status.StatusCode,
			&user_status.Name,
			&user_status.Description,
			&user_status.IsActive,
			&user_status.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"User_statusRepository.repository.GetUser_statusBystatuscode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return user_status, nil
}
func (t User_statusRepository)  DeleteUser_statusById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_statusRepository -> DeleteUser_statusById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteUser_statusById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_USER_STATUS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"User_statusRepository.repository.DeleteUser_statusById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteUser_statusById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteUser_statusById.deleted", result)
	return true, err
}
func (t User_statusRepository)  InsertUser_status(ctx context.Context,user_status *model.User_status) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_statusRepository -> InsertUser_status", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertUser_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertUser_status.statuscode", user_status.StatusCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_USER_STATUS_INSERT,
			user_status.StatusCode,
			user_status.Name,
			user_status.Description,
			user_status.IsActive,
			user_status.CreatedAt,
	).Scan(&user_status.ID)

	if err != nil {
		t.log.Error(ctx, "User_statusRepository.repository.InsertUser_status.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertUser_status.inserted_id", user_status.ID)
   return user_status.ID, nil

}
func (t User_statusRepository)  UpdateUser_status(ctx context.Context,user_status *model.User_status, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_statusRepository -> UpdateUser_status", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateUser_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateUser_status.id", id)
	tracker.AddParam("repository.UpdateUser_status.statuscode", user_status.StatusCode)

	user_status.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_USER_STATUS_UPDATE, 
			user_status.StatusCode,
			user_status.Name,
			user_status.Description,
			user_status.IsActive,
			user_status.CreatedAt,
			user_status.ID,
   )
	if err != nil {
		t.log.Error(ctx, "User_statusRepository.repository.UpdateUser_status.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateUser_status.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateUser_status.rows_affected", rowsAffected)
	return nil
}

