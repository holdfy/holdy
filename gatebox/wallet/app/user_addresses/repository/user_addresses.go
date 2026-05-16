package user_addressesRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type User_addressesRepositoryIF interface {
     GetUser_addresses(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetUser_addressesById(ctx context.Context, id int64) (*model.User_addresses, error)
     GetUser_addressesByAddressCode(ctx context.Context, addresscode string) (*model.User_addresses, error)
     InsertUser_addresses(ctx context.Context, user_addresses *model.User_addresses) (int64, error)
     UpdateUser_addresses(ctx context.Context, user_addresses *model.User_addresses, id int64) error
     DeleteUser_addressesById(ctx context.Context, id int64) (bool, error)
}
 type User_addressesRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewUser_addressesRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *User_addressesRepository{
    return &User_addressesRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("User_addresses"),
     }
}
func (t User_addressesRepository)  GetUser_addresses(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_addressesRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_addresses")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_addresses.offset", offset)
	tracker.AddParam("repository.GetUser_addresses.limit", limit)
	itemsPage 			= model.ItemsPage{}
	user_addressess := []model.User_addresses{}

	rows, err := t.PGRead.Query(ctx, SQL_USER_ADDRESSES_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "User_addressesRepository.repository.GetUser_addressess.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var user_addresses model.User_addresses
		err := rows.Scan(
			&user_addresses.ID,
			&user_addresses.AddressCode,
			&user_addresses.UserId,
			&user_addresses.IdAddressType,
			&user_addresses.Street,
			&user_addresses.Number,
			&user_addresses.Complement,
			&user_addresses.Neighborhood,
			&user_addresses.City,
			&user_addresses.State,
			&user_addresses.ZipCode,
			&user_addresses.Country,
			&user_addresses.IsPrimary,
			&user_addresses.IsActive,
			&user_addresses.CreatedAt,
			&user_addresses.UpdatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "User_addressesRepository.repository.GetUser_addressess.Scan: ", err.Error())
			return itemsPage, err
		}
		user_addressess = append(user_addressess, user_addresses)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "User_addressesRepository.repository.GetUser_addressess.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(user_addressess) > 0 {
		qtyRecords = user_addressess[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = user_addressess

	tracker.AddResult("repository.GetUser_addresses.rows_returned", len(user_addressess))
	tracker.AddResult("repository.GetUser_addresses.total_count", len(user_addressess))

	return itemsPage, nil
}
func (t User_addressesRepository)  GetUser_addressesById(ctx context.Context, id int64) (user_addresses *model.User_addresses, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_addressesRepository -> GetUser_addressesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_addressesById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_addressesById.id", id)

	user_addresses = new(model.User_addresses)
	row := t.PGRead.QueryRow(ctx, SQL_GET_USER_ADDRESSES_BY_ID, id)
		err = row.Scan(
			&user_addresses.ID,
			&user_addresses.AddressCode,
			&user_addresses.UserId,
			&user_addresses.IdAddressType,
			&user_addresses.Street,
			&user_addresses.Number,
			&user_addresses.Complement,
			&user_addresses.Neighborhood,
			&user_addresses.City,
			&user_addresses.State,
			&user_addresses.ZipCode,
			&user_addresses.Country,
			&user_addresses.IsPrimary,
			&user_addresses.IsActive,
			&user_addresses.CreatedAt,
			&user_addresses.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"User_addressesRepository.repository.GetUser_addressesById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetUser_addressesById.found", true)
	return user_addresses, nil
}
func (t User_addressesRepository)  GetUser_addressesByAddressCode(ctx context.Context, addresscode string) (user_addresses *model.User_addresses, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_addressesRepository -> GetUser_addressesByAddressCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_addressesByAddressCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_addressesByAddressCode.addresscode", addresscode)

	user_addresses = new(model.User_addresses)
	row := t.PGRead.QueryRow(ctx, SQL_GET_USER_ADDRESSES_BY_ADDRESS_CODE, addresscode)
		err = row.Scan(
			&user_addresses.ID,
			&user_addresses.AddressCode,
			&user_addresses.UserId,
			&user_addresses.IdAddressType,
			&user_addresses.Street,
			&user_addresses.Number,
			&user_addresses.Complement,
			&user_addresses.Neighborhood,
			&user_addresses.City,
			&user_addresses.State,
			&user_addresses.ZipCode,
			&user_addresses.Country,
			&user_addresses.IsPrimary,
			&user_addresses.IsActive,
			&user_addresses.CreatedAt,
			&user_addresses.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"User_addressesRepository.repository.GetUser_addressesByaddresscode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return user_addresses, nil
}
func (t User_addressesRepository)  DeleteUser_addressesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_addressesRepository -> DeleteUser_addressesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteUser_addressesById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_USER_ADDRESSES_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"User_addressesRepository.repository.DeleteUser_addressesById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteUser_addressesById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteUser_addressesById.deleted", result)
	return true, err
}
func (t User_addressesRepository)  InsertUser_addresses(ctx context.Context,user_addresses *model.User_addresses) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_addressesRepository -> InsertUser_addresses", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertUser_addresses")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertUser_addresses.addresscode", user_addresses.AddressCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_USER_ADDRESSES_INSERT,
			user_addresses.AddressCode,
			user_addresses.UserId,
			user_addresses.IdAddressType,
			user_addresses.Street,
			user_addresses.Number,
			user_addresses.Complement,
			user_addresses.Neighborhood,
			user_addresses.City,
			user_addresses.State,
			user_addresses.ZipCode,
			user_addresses.Country,
			user_addresses.IsPrimary,
			user_addresses.IsActive,
			user_addresses.CreatedAt,
			user_addresses.UpdatedAt,
	).Scan(&user_addresses.ID)

	if err != nil {
		t.log.Error(ctx, "User_addressesRepository.repository.InsertUser_addresses.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertUser_addresses.inserted_id", user_addresses.ID)
   return user_addresses.ID, nil

}
func (t User_addressesRepository)  UpdateUser_addresses(ctx context.Context,user_addresses *model.User_addresses, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_addressesRepository -> UpdateUser_addresses", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateUser_addresses")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateUser_addresses.id", id)
	tracker.AddParam("repository.UpdateUser_addresses.addresscode", user_addresses.AddressCode)

	user_addresses.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_USER_ADDRESSES_UPDATE, 
			user_addresses.AddressCode,
			user_addresses.UserId,
			user_addresses.IdAddressType,
			user_addresses.Street,
			user_addresses.Number,
			user_addresses.Complement,
			user_addresses.Neighborhood,
			user_addresses.City,
			user_addresses.State,
			user_addresses.ZipCode,
			user_addresses.Country,
			user_addresses.IsPrimary,
			user_addresses.IsActive,
			user_addresses.CreatedAt,
			user_addresses.UpdatedAt,
			user_addresses.ID,
   )
	if err != nil {
		t.log.Error(ctx, "User_addressesRepository.repository.UpdateUser_addresses.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateUser_addresses.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateUser_addresses.rows_affected", rowsAffected)
	return nil
}

