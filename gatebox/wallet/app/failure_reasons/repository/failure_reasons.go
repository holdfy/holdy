package failure_reasonsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Failure_reasonsRepositoryIF interface {
     GetFailure_reasons(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetFailure_reasonsById(ctx context.Context, id int64) (*model.Failure_reasons, error)
     GetFailure_reasonsByReasonCode(ctx context.Context, reasoncode string) (*model.Failure_reasons, error)
     InsertFailure_reasons(ctx context.Context, failure_reasons *model.Failure_reasons) (int64, error)
     UpdateFailure_reasons(ctx context.Context, failure_reasons *model.Failure_reasons, id int64) error
     DeleteFailure_reasonsById(ctx context.Context, id int64) (bool, error)
}
 type Failure_reasonsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewFailure_reasonsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Failure_reasonsRepository{
    return &Failure_reasonsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Failure_reasons"),
     }
}
func (t Failure_reasonsRepository)  GetFailure_reasons(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Failure_reasonsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetFailure_reasons")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetFailure_reasons.offset", offset)
	tracker.AddParam("repository.GetFailure_reasons.limit", limit)
	itemsPage 			= model.ItemsPage{}
	failure_reasonss := []model.Failure_reasons{}

	rows, err := t.PGRead.Query(ctx, SQL_FAILURE_REASONS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Failure_reasonsRepository.repository.GetFailure_reasonss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var failure_reasons model.Failure_reasons
		err := rows.Scan(
			&failure_reasons.ID,
			&failure_reasons.ReasonCode,
			&failure_reasons.Name,
			&failure_reasons.Description,
			&failure_reasons.IsCritical,
			&failure_reasons.IsActive,
			&failure_reasons.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Failure_reasonsRepository.repository.GetFailure_reasonss.Scan: ", err.Error())
			return itemsPage, err
		}
		failure_reasonss = append(failure_reasonss, failure_reasons)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Failure_reasonsRepository.repository.GetFailure_reasonss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(failure_reasonss) > 0 {
		qtyRecords = failure_reasonss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = failure_reasonss

	tracker.AddResult("repository.GetFailure_reasons.rows_returned", len(failure_reasonss))
	tracker.AddResult("repository.GetFailure_reasons.total_count", len(failure_reasonss))

	return itemsPage, nil
}
func (t Failure_reasonsRepository)  GetFailure_reasonsById(ctx context.Context, id int64) (failure_reasons *model.Failure_reasons, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Failure_reasonsRepository -> GetFailure_reasonsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetFailure_reasonsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetFailure_reasonsById.id", id)

	failure_reasons = new(model.Failure_reasons)
	row := t.PGRead.QueryRow(ctx, SQL_GET_FAILURE_REASONS_BY_ID, id)
		err = row.Scan(
			&failure_reasons.ID,
			&failure_reasons.ReasonCode,
			&failure_reasons.Name,
			&failure_reasons.Description,
			&failure_reasons.IsCritical,
			&failure_reasons.IsActive,
			&failure_reasons.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Failure_reasonsRepository.repository.GetFailure_reasonsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetFailure_reasonsById.found", true)
	return failure_reasons, nil
}
func (t Failure_reasonsRepository)  GetFailure_reasonsByReasonCode(ctx context.Context, reasoncode string) (failure_reasons *model.Failure_reasons, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Failure_reasonsRepository -> GetFailure_reasonsByReasonCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetFailure_reasonsByReasonCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetFailure_reasonsByReasonCode.reasoncode", reasoncode)

	failure_reasons = new(model.Failure_reasons)
	row := t.PGRead.QueryRow(ctx, SQL_GET_FAILURE_REASONS_BY_REASON_CODE, reasoncode)
		err = row.Scan(
			&failure_reasons.ID,
			&failure_reasons.ReasonCode,
			&failure_reasons.Name,
			&failure_reasons.Description,
			&failure_reasons.IsCritical,
			&failure_reasons.IsActive,
			&failure_reasons.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Failure_reasonsRepository.repository.GetFailure_reasonsByreasoncode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return failure_reasons, nil
}
func (t Failure_reasonsRepository)  DeleteFailure_reasonsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Failure_reasonsRepository -> DeleteFailure_reasonsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteFailure_reasonsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_FAILURE_REASONS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Failure_reasonsRepository.repository.DeleteFailure_reasonsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteFailure_reasonsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteFailure_reasonsById.deleted", result)
	return true, err
}
func (t Failure_reasonsRepository)  InsertFailure_reasons(ctx context.Context,failure_reasons *model.Failure_reasons) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Failure_reasonsRepository -> InsertFailure_reasons", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertFailure_reasons")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertFailure_reasons.reasoncode", failure_reasons.ReasonCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_FAILURE_REASONS_INSERT,
			failure_reasons.ReasonCode,
			failure_reasons.Name,
			failure_reasons.Description,
			failure_reasons.IsCritical,
			failure_reasons.IsActive,
			failure_reasons.CreatedAt,
	).Scan(&failure_reasons.ID)

	if err != nil {
		t.log.Error(ctx, "Failure_reasonsRepository.repository.InsertFailure_reasons.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertFailure_reasons.inserted_id", failure_reasons.ID)
   return failure_reasons.ID, nil

}
func (t Failure_reasonsRepository)  UpdateFailure_reasons(ctx context.Context,failure_reasons *model.Failure_reasons, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Failure_reasonsRepository -> UpdateFailure_reasons", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateFailure_reasons")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateFailure_reasons.id", id)
	tracker.AddParam("repository.UpdateFailure_reasons.reasoncode", failure_reasons.ReasonCode)

	failure_reasons.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_FAILURE_REASONS_UPDATE, 
			failure_reasons.ReasonCode,
			failure_reasons.Name,
			failure_reasons.Description,
			failure_reasons.IsCritical,
			failure_reasons.IsActive,
			failure_reasons.CreatedAt,
			failure_reasons.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Failure_reasonsRepository.repository.UpdateFailure_reasons.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateFailure_reasons.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateFailure_reasons.rows_affected", rowsAffected)
	return nil
}

