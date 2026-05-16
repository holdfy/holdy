package user_restrictionsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type User_restrictionsRepositoryIF interface {
     GetUser_restrictions(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetUser_restrictionsById(ctx context.Context, id int64) (*model.User_restrictions, error)
     GetUser_restrictionsByRestrictionCode(ctx context.Context, restrictioncode string) (*model.User_restrictions, error)
     InsertUser_restrictions(ctx context.Context, user_restrictions *model.User_restrictions) (int64, error)
     UpdateUser_restrictions(ctx context.Context, user_restrictions *model.User_restrictions, id int64) error
     DeleteUser_restrictionsById(ctx context.Context, id int64) (bool, error)
}
 type User_restrictionsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewUser_restrictionsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *User_restrictionsRepository{
    return &User_restrictionsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("User_restrictions"),
     }
}
func (t User_restrictionsRepository)  GetUser_restrictions(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_restrictionsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_restrictions")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_restrictions.offset", offset)
	tracker.AddParam("repository.GetUser_restrictions.limit", limit)
	itemsPage 			= model.ItemsPage{}
	user_restrictionss := []model.User_restrictions{}

	rows, err := t.PGRead.Query(ctx, SQL_USER_RESTRICTIONS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "User_restrictionsRepository.repository.GetUser_restrictionss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var user_restrictions model.User_restrictions
		err := rows.Scan(
			&user_restrictions.ID,
			&user_restrictions.RestrictionCode,
			&user_restrictions.UserId,
			&user_restrictions.IdRestrictionType,
			&user_restrictions.RestrictionReason,
			&user_restrictions.Restrictions,
			&user_restrictions.IsActive,
			&user_restrictions.ExpiresAt,
			&user_restrictions.CreatedBy,
			&user_restrictions.RemovedBy,
			&user_restrictions.RemovedAt,
			&user_restrictions.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "User_restrictionsRepository.repository.GetUser_restrictionss.Scan: ", err.Error())
			return itemsPage, err
		}
		user_restrictionss = append(user_restrictionss, user_restrictions)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "User_restrictionsRepository.repository.GetUser_restrictionss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(user_restrictionss) > 0 {
		qtyRecords = user_restrictionss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = user_restrictionss

	tracker.AddResult("repository.GetUser_restrictions.rows_returned", len(user_restrictionss))
	tracker.AddResult("repository.GetUser_restrictions.total_count", len(user_restrictionss))

	return itemsPage, nil
}
func (t User_restrictionsRepository)  GetUser_restrictionsById(ctx context.Context, id int64) (user_restrictions *model.User_restrictions, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_restrictionsRepository -> GetUser_restrictionsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_restrictionsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_restrictionsById.id", id)

	user_restrictions = new(model.User_restrictions)
	row := t.PGRead.QueryRow(ctx, SQL_GET_USER_RESTRICTIONS_BY_ID, id)
		err = row.Scan(
			&user_restrictions.ID,
			&user_restrictions.RestrictionCode,
			&user_restrictions.UserId,
			&user_restrictions.IdRestrictionType,
			&user_restrictions.RestrictionReason,
			&user_restrictions.Restrictions,
			&user_restrictions.IsActive,
			&user_restrictions.ExpiresAt,
			&user_restrictions.CreatedBy,
			&user_restrictions.RemovedBy,
			&user_restrictions.RemovedAt,
			&user_restrictions.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"User_restrictionsRepository.repository.GetUser_restrictionsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetUser_restrictionsById.found", true)
	return user_restrictions, nil
}
func (t User_restrictionsRepository)  GetUser_restrictionsByRestrictionCode(ctx context.Context, restrictioncode string) (user_restrictions *model.User_restrictions, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_restrictionsRepository -> GetUser_restrictionsByRestrictionCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_restrictionsByRestrictionCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_restrictionsByRestrictionCode.restrictioncode", restrictioncode)

	user_restrictions = new(model.User_restrictions)
	row := t.PGRead.QueryRow(ctx, SQL_GET_USER_RESTRICTIONS_BY_RESTRICTION_CODE, restrictioncode)
		err = row.Scan(
			&user_restrictions.ID,
			&user_restrictions.RestrictionCode,
			&user_restrictions.UserId,
			&user_restrictions.IdRestrictionType,
			&user_restrictions.RestrictionReason,
			&user_restrictions.Restrictions,
			&user_restrictions.IsActive,
			&user_restrictions.ExpiresAt,
			&user_restrictions.CreatedBy,
			&user_restrictions.RemovedBy,
			&user_restrictions.RemovedAt,
			&user_restrictions.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"User_restrictionsRepository.repository.GetUser_restrictionsByrestrictioncode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return user_restrictions, nil
}
func (t User_restrictionsRepository)  DeleteUser_restrictionsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_restrictionsRepository -> DeleteUser_restrictionsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteUser_restrictionsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_USER_RESTRICTIONS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"User_restrictionsRepository.repository.DeleteUser_restrictionsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteUser_restrictionsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteUser_restrictionsById.deleted", result)
	return true, err
}
func (t User_restrictionsRepository)  InsertUser_restrictions(ctx context.Context,user_restrictions *model.User_restrictions) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_restrictionsRepository -> InsertUser_restrictions", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertUser_restrictions")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertUser_restrictions.restrictioncode", user_restrictions.RestrictionCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_USER_RESTRICTIONS_INSERT,
			user_restrictions.RestrictionCode,
			user_restrictions.UserId,
			user_restrictions.IdRestrictionType,
			user_restrictions.RestrictionReason,
			user_restrictions.Restrictions,
			user_restrictions.IsActive,
			user_restrictions.ExpiresAt,
			user_restrictions.CreatedBy,
			user_restrictions.RemovedBy,
			user_restrictions.RemovedAt,
			user_restrictions.CreatedAt,
	).Scan(&user_restrictions.ID)

	if err != nil {
		t.log.Error(ctx, "User_restrictionsRepository.repository.InsertUser_restrictions.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertUser_restrictions.inserted_id", user_restrictions.ID)
   return user_restrictions.ID, nil

}
func (t User_restrictionsRepository)  UpdateUser_restrictions(ctx context.Context,user_restrictions *model.User_restrictions, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_restrictionsRepository -> UpdateUser_restrictions", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateUser_restrictions")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateUser_restrictions.id", id)
	tracker.AddParam("repository.UpdateUser_restrictions.restrictioncode", user_restrictions.RestrictionCode)

	user_restrictions.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_USER_RESTRICTIONS_UPDATE, 
			user_restrictions.RestrictionCode,
			user_restrictions.UserId,
			user_restrictions.IdRestrictionType,
			user_restrictions.RestrictionReason,
			user_restrictions.Restrictions,
			user_restrictions.IsActive,
			user_restrictions.ExpiresAt,
			user_restrictions.CreatedBy,
			user_restrictions.RemovedBy,
			user_restrictions.RemovedAt,
			user_restrictions.CreatedAt,
			user_restrictions.ID,
   )
	if err != nil {
		t.log.Error(ctx, "User_restrictionsRepository.repository.UpdateUser_restrictions.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateUser_restrictions.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateUser_restrictions.rows_affected", rowsAffected)
	return nil
}

