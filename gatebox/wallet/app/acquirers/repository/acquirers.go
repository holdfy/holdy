package acquirersRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type AcquirersRepositoryIF interface {
     GetAcquirers(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetAcquirersById(ctx context.Context, id int64) (*model.Acquirers, error)
     GetAcquirersByAcquirerCode(ctx context.Context, acquirercode string) (*model.Acquirers, error)
     InsertAcquirers(ctx context.Context, acquirers *model.Acquirers) (int64, error)
     UpdateAcquirers(ctx context.Context, acquirers *model.Acquirers, id int64) error
     DeleteAcquirersById(ctx context.Context, id int64) (bool, error)
}
 type AcquirersRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewAcquirersRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *AcquirersRepository{
    return &AcquirersRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Acquirers"),
     }
}
func (t AcquirersRepository)  GetAcquirers(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("AcquirersRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAcquirers")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAcquirers.offset", offset)
	tracker.AddParam("repository.GetAcquirers.limit", limit)
	itemsPage 			= model.ItemsPage{}
	acquirerss := []model.Acquirers{}

	rows, err := t.PGRead.Query(ctx, SQL_ACQUIRERS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "AcquirersRepository.repository.GetAcquirerss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var acquirers model.Acquirers
		err := rows.Scan(
			&acquirers.ID,
			&acquirers.AcquirerCode,
			&acquirers.Name,
			&acquirers.Description,
			&acquirers.ApiEndpoint,
			&acquirers.IsActive,
			&acquirers.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "AcquirersRepository.repository.GetAcquirerss.Scan: ", err.Error())
			return itemsPage, err
		}
		acquirerss = append(acquirerss, acquirers)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "AcquirersRepository.repository.GetAcquirerss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(acquirerss) > 0 {
		qtyRecords = acquirerss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = acquirerss

	tracker.AddResult("repository.GetAcquirers.rows_returned", len(acquirerss))
	tracker.AddResult("repository.GetAcquirers.total_count", len(acquirerss))

	return itemsPage, nil
}
func (t AcquirersRepository)  GetAcquirersById(ctx context.Context, id int64) (acquirers *model.Acquirers, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("AcquirersRepository -> GetAcquirersById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAcquirersById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAcquirersById.id", id)

	acquirers = new(model.Acquirers)
	row := t.PGRead.QueryRow(ctx, SQL_GET_ACQUIRERS_BY_ID, id)
		err = row.Scan(
			&acquirers.ID,
			&acquirers.AcquirerCode,
			&acquirers.Name,
			&acquirers.Description,
			&acquirers.ApiEndpoint,
			&acquirers.IsActive,
			&acquirers.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"AcquirersRepository.repository.GetAcquirersById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetAcquirersById.found", true)
	return acquirers, nil
}
func (t AcquirersRepository)  GetAcquirersByAcquirerCode(ctx context.Context, acquirercode string) (acquirers *model.Acquirers, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("AcquirersRepository -> GetAcquirersByAcquirerCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAcquirersByAcquirerCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAcquirersByAcquirerCode.acquirercode", acquirercode)

	acquirers = new(model.Acquirers)
	row := t.PGRead.QueryRow(ctx, SQL_GET_ACQUIRERS_BY_ACQUIRER_CODE, acquirercode)
		err = row.Scan(
			&acquirers.ID,
			&acquirers.AcquirerCode,
			&acquirers.Name,
			&acquirers.Description,
			&acquirers.ApiEndpoint,
			&acquirers.IsActive,
			&acquirers.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"AcquirersRepository.repository.GetAcquirersByacquirercode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return acquirers, nil
}
func (t AcquirersRepository)  DeleteAcquirersById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("AcquirersRepository -> DeleteAcquirersById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteAcquirersById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_ACQUIRERS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"AcquirersRepository.repository.DeleteAcquirersById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteAcquirersById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteAcquirersById.deleted", result)
	return true, err
}
func (t AcquirersRepository)  InsertAcquirers(ctx context.Context,acquirers *model.Acquirers) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("AcquirersRepository -> InsertAcquirers", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertAcquirers")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertAcquirers.acquirercode", acquirers.AcquirerCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_ACQUIRERS_INSERT,
			acquirers.AcquirerCode,
			acquirers.Name,
			acquirers.Description,
			acquirers.ApiEndpoint,
			acquirers.IsActive,
			acquirers.CreatedAt,
	).Scan(&acquirers.ID)

	if err != nil {
		t.log.Error(ctx, "AcquirersRepository.repository.InsertAcquirers.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertAcquirers.inserted_id", acquirers.ID)
   return acquirers.ID, nil

}
func (t AcquirersRepository)  UpdateAcquirers(ctx context.Context,acquirers *model.Acquirers, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("AcquirersRepository -> UpdateAcquirers", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateAcquirers")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateAcquirers.id", id)
	tracker.AddParam("repository.UpdateAcquirers.acquirercode", acquirers.AcquirerCode)

	acquirers.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_ACQUIRERS_UPDATE, 
			acquirers.AcquirerCode,
			acquirers.Name,
			acquirers.Description,
			acquirers.ApiEndpoint,
			acquirers.IsActive,
			acquirers.CreatedAt,
			acquirers.ID,
   )
	if err != nil {
		t.log.Error(ctx, "AcquirersRepository.repository.UpdateAcquirers.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateAcquirers.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateAcquirers.rows_affected", rowsAffected)
	return nil
}

