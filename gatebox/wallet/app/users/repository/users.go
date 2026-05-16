package usersRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type UsersRepositoryIF interface {
     GetUsers(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetUsersById(ctx context.Context, id int64) (*model.Users, error)
     GetUsersByUserCode(ctx context.Context, usercode string) (*model.Users, error)
     InsertUsers(ctx context.Context, users *model.Users) (int64, error)
     UpdateUsers(ctx context.Context, users *model.Users, id int64) error
     DeleteUsersById(ctx context.Context, id int64) (bool, error)
}
 type UsersRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewUsersRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *UsersRepository{
    return &UsersRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Users"),
     }
}
func (t UsersRepository)  GetUsers(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("UsersRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUsers")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUsers.offset", offset)
	tracker.AddParam("repository.GetUsers.limit", limit)
	itemsPage 			= model.ItemsPage{}
	userss := []model.Users{}

	rows, err := t.PGRead.Query(ctx, SQL_USERS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "UsersRepository.repository.GetUserss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var users model.Users
		err := rows.Scan(
			&users.ID,
			&users.UserCode,
			&users.Cpf,
			&users.FullName,
			&users.Email,
			&users.Phone,
			&users.BirthDate,
			&users.IdStatus,
			&users.IdKycStatus,
			&users.KycLevel,
			&users.AppPasswordHash,
			&users.BiometricFailures,
			&users.LastLogin,
			&users.CreatedAt,
			&users.UpdatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "UsersRepository.repository.GetUserss.Scan: ", err.Error())
			return itemsPage, err
		}
		userss = append(userss, users)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "UsersRepository.repository.GetUserss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(userss) > 0 {
		qtyRecords = userss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = userss

	tracker.AddResult("repository.GetUsers.rows_returned", len(userss))
	tracker.AddResult("repository.GetUsers.total_count", len(userss))

	return itemsPage, nil
}
func (t UsersRepository)  GetUsersById(ctx context.Context, id int64) (users *model.Users, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("UsersRepository -> GetUsersById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUsersById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUsersById.id", id)

	users = new(model.Users)
	row := t.PGRead.QueryRow(ctx, SQL_GET_USERS_BY_ID, id)
		err = row.Scan(
			&users.ID,
			&users.UserCode,
			&users.Cpf,
			&users.FullName,
			&users.Email,
			&users.Phone,
			&users.BirthDate,
			&users.IdStatus,
			&users.IdKycStatus,
			&users.KycLevel,
			&users.AppPasswordHash,
			&users.BiometricFailures,
			&users.LastLogin,
			&users.CreatedAt,
			&users.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"UsersRepository.repository.GetUsersById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetUsersById.found", true)
	return users, nil
}
func (t UsersRepository)  GetUsersByUserCode(ctx context.Context, usercode string) (users *model.Users, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("UsersRepository -> GetUsersByUserCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUsersByUserCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUsersByUserCode.usercode", usercode)

	users = new(model.Users)
	row := t.PGRead.QueryRow(ctx, SQL_GET_USERS_BY_USER_CODE, usercode)
		err = row.Scan(
			&users.ID,
			&users.UserCode,
			&users.Cpf,
			&users.FullName,
			&users.Email,
			&users.Phone,
			&users.BirthDate,
			&users.IdStatus,
			&users.IdKycStatus,
			&users.KycLevel,
			&users.AppPasswordHash,
			&users.BiometricFailures,
			&users.LastLogin,
			&users.CreatedAt,
			&users.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"UsersRepository.repository.GetUsersByusercode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return users, nil
}
func (t UsersRepository)  DeleteUsersById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("UsersRepository -> DeleteUsersById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteUsersById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_USERS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"UsersRepository.repository.DeleteUsersById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteUsersById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteUsersById.deleted", result)
	return true, err
}
func (t UsersRepository)  InsertUsers(ctx context.Context,users *model.Users) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("UsersRepository -> InsertUsers", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertUsers")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertUsers.usercode", users.UserCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_USERS_INSERT,
			users.UserCode,
			users.Cpf,
			users.FullName,
			users.Email,
			users.Phone,
			users.BirthDate,
			users.IdStatus,
			users.IdKycStatus,
			users.KycLevel,
			users.AppPasswordHash,
			users.BiometricFailures,
			users.LastLogin,
			users.CreatedAt,
			users.UpdatedAt,
	).Scan(&users.ID)

	if err != nil {
		t.log.Error(ctx, "UsersRepository.repository.InsertUsers.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertUsers.inserted_id", users.ID)
   return users.ID, nil

}
func (t UsersRepository)  UpdateUsers(ctx context.Context,users *model.Users, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("UsersRepository -> UpdateUsers", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateUsers")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateUsers.id", id)
	tracker.AddParam("repository.UpdateUsers.usercode", users.UserCode)

	users.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_USERS_UPDATE, 
			users.UserCode,
			users.Cpf,
			users.FullName,
			users.Email,
			users.Phone,
			users.BirthDate,
			users.IdStatus,
			users.IdKycStatus,
			users.KycLevel,
			users.AppPasswordHash,
			users.BiometricFailures,
			users.LastLogin,
			users.CreatedAt,
			users.UpdatedAt,
			users.ID,
   )
	if err != nil {
		t.log.Error(ctx, "UsersRepository.repository.UpdateUsers.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateUsers.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateUsers.rows_affected", rowsAffected)
	return nil
}

