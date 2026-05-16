package attempt_resultsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Attempt_resultsRepositoryIF interface {
     GetAttempt_results(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetAttempt_resultsById(ctx context.Context, id int64) (*model.Attempt_results, error)
     GetAttempt_resultsByResultCode(ctx context.Context, resultcode string) (*model.Attempt_results, error)
     InsertAttempt_results(ctx context.Context, attempt_results *model.Attempt_results) (int64, error)
     UpdateAttempt_results(ctx context.Context, attempt_results *model.Attempt_results, id int64) error
     DeleteAttempt_resultsById(ctx context.Context, id int64) (bool, error)
}
 type Attempt_resultsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewAttempt_resultsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Attempt_resultsRepository{
    return &Attempt_resultsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Attempt_results"),
     }
}
func (t Attempt_resultsRepository)  GetAttempt_results(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Attempt_resultsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAttempt_results")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAttempt_results.offset", offset)
	tracker.AddParam("repository.GetAttempt_results.limit", limit)
	itemsPage 			= model.ItemsPage{}
	attempt_resultss := []model.Attempt_results{}

	rows, err := t.PGRead.Query(ctx, SQL_ATTEMPT_RESULTS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Attempt_resultsRepository.repository.GetAttempt_resultss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var attempt_results model.Attempt_results
		err := rows.Scan(
			&attempt_results.ID,
			&attempt_results.ResultCode,
			&attempt_results.Name,
			&attempt_results.Description,
			&attempt_results.IsSuccess,
			&attempt_results.IsActive,
			&attempt_results.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Attempt_resultsRepository.repository.GetAttempt_resultss.Scan: ", err.Error())
			return itemsPage, err
		}
		attempt_resultss = append(attempt_resultss, attempt_results)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Attempt_resultsRepository.repository.GetAttempt_resultss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(attempt_resultss) > 0 {
		qtyRecords = attempt_resultss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = attempt_resultss

	tracker.AddResult("repository.GetAttempt_results.rows_returned", len(attempt_resultss))
	tracker.AddResult("repository.GetAttempt_results.total_count", len(attempt_resultss))

	return itemsPage, nil
}
func (t Attempt_resultsRepository)  GetAttempt_resultsById(ctx context.Context, id int64) (attempt_results *model.Attempt_results, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Attempt_resultsRepository -> GetAttempt_resultsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAttempt_resultsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAttempt_resultsById.id", id)

	attempt_results = new(model.Attempt_results)
	row := t.PGRead.QueryRow(ctx, SQL_GET_ATTEMPT_RESULTS_BY_ID, id)
		err = row.Scan(
			&attempt_results.ID,
			&attempt_results.ResultCode,
			&attempt_results.Name,
			&attempt_results.Description,
			&attempt_results.IsSuccess,
			&attempt_results.IsActive,
			&attempt_results.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Attempt_resultsRepository.repository.GetAttempt_resultsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetAttempt_resultsById.found", true)
	return attempt_results, nil
}
func (t Attempt_resultsRepository)  GetAttempt_resultsByResultCode(ctx context.Context, resultcode string) (attempt_results *model.Attempt_results, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Attempt_resultsRepository -> GetAttempt_resultsByResultCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAttempt_resultsByResultCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAttempt_resultsByResultCode.resultcode", resultcode)

	attempt_results = new(model.Attempt_results)
	row := t.PGRead.QueryRow(ctx, SQL_GET_ATTEMPT_RESULTS_BY_RESULT_CODE, resultcode)
		err = row.Scan(
			&attempt_results.ID,
			&attempt_results.ResultCode,
			&attempt_results.Name,
			&attempt_results.Description,
			&attempt_results.IsSuccess,
			&attempt_results.IsActive,
			&attempt_results.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Attempt_resultsRepository.repository.GetAttempt_resultsByresultcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return attempt_results, nil
}
func (t Attempt_resultsRepository)  DeleteAttempt_resultsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Attempt_resultsRepository -> DeleteAttempt_resultsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteAttempt_resultsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_ATTEMPT_RESULTS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Attempt_resultsRepository.repository.DeleteAttempt_resultsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteAttempt_resultsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteAttempt_resultsById.deleted", result)
	return true, err
}
func (t Attempt_resultsRepository)  InsertAttempt_results(ctx context.Context,attempt_results *model.Attempt_results) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Attempt_resultsRepository -> InsertAttempt_results", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertAttempt_results")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertAttempt_results.resultcode", attempt_results.ResultCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_ATTEMPT_RESULTS_INSERT,
			attempt_results.ResultCode,
			attempt_results.Name,
			attempt_results.Description,
			attempt_results.IsSuccess,
			attempt_results.IsActive,
			attempt_results.CreatedAt,
	).Scan(&attempt_results.ID)

	if err != nil {
		t.log.Error(ctx, "Attempt_resultsRepository.repository.InsertAttempt_results.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertAttempt_results.inserted_id", attempt_results.ID)
   return attempt_results.ID, nil

}
func (t Attempt_resultsRepository)  UpdateAttempt_results(ctx context.Context,attempt_results *model.Attempt_results, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Attempt_resultsRepository -> UpdateAttempt_results", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateAttempt_results")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateAttempt_results.id", id)
	tracker.AddParam("repository.UpdateAttempt_results.resultcode", attempt_results.ResultCode)

	attempt_results.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_ATTEMPT_RESULTS_UPDATE, 
			attempt_results.ResultCode,
			attempt_results.Name,
			attempt_results.Description,
			attempt_results.IsSuccess,
			attempt_results.IsActive,
			attempt_results.CreatedAt,
			attempt_results.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Attempt_resultsRepository.repository.UpdateAttempt_results.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateAttempt_results.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateAttempt_results.rows_affected", rowsAffected)
	return nil
}

