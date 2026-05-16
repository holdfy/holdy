package kyc_statusRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Kyc_statusRepositoryIF interface {
     GetKyc_status(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetKyc_statusById(ctx context.Context, id int64) (*model.Kyc_status, error)
     GetKyc_statusByStatusCode(ctx context.Context, statuscode string) (*model.Kyc_status, error)
     InsertKyc_status(ctx context.Context, kyc_status *model.Kyc_status) (int64, error)
     UpdateKyc_status(ctx context.Context, kyc_status *model.Kyc_status, id int64) error
     DeleteKyc_statusById(ctx context.Context, id int64) (bool, error)
}
 type Kyc_statusRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewKyc_statusRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Kyc_statusRepository{
    return &Kyc_statusRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Kyc_status"),
     }
}
func (t Kyc_statusRepository)  GetKyc_status(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Kyc_statusRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetKyc_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetKyc_status.offset", offset)
	tracker.AddParam("repository.GetKyc_status.limit", limit)
	itemsPage 			= model.ItemsPage{}
	kyc_statuss := []model.Kyc_status{}

	rows, err := t.PGRead.Query(ctx, SQL_KYC_STATUS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Kyc_statusRepository.repository.GetKyc_statuss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var kyc_status model.Kyc_status
		err := rows.Scan(
			&kyc_status.ID,
			&kyc_status.StatusCode,
			&kyc_status.Name,
			&kyc_status.Description,
			&kyc_status.AllowsTransactions,
			&kyc_status.MaxTransactionAmount,
			&kyc_status.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Kyc_statusRepository.repository.GetKyc_statuss.Scan: ", err.Error())
			return itemsPage, err
		}
		kyc_statuss = append(kyc_statuss, kyc_status)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Kyc_statusRepository.repository.GetKyc_statuss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(kyc_statuss) > 0 {
		qtyRecords = kyc_statuss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = kyc_statuss

	tracker.AddResult("repository.GetKyc_status.rows_returned", len(kyc_statuss))
	tracker.AddResult("repository.GetKyc_status.total_count", len(kyc_statuss))

	return itemsPage, nil
}
func (t Kyc_statusRepository)  GetKyc_statusById(ctx context.Context, id int64) (kyc_status *model.Kyc_status, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Kyc_statusRepository -> GetKyc_statusById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetKyc_statusById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetKyc_statusById.id", id)

	kyc_status = new(model.Kyc_status)
	row := t.PGRead.QueryRow(ctx, SQL_GET_KYC_STATUS_BY_ID, id)
		err = row.Scan(
			&kyc_status.ID,
			&kyc_status.StatusCode,
			&kyc_status.Name,
			&kyc_status.Description,
			&kyc_status.AllowsTransactions,
			&kyc_status.MaxTransactionAmount,
			&kyc_status.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Kyc_statusRepository.repository.GetKyc_statusById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetKyc_statusById.found", true)
	return kyc_status, nil
}
func (t Kyc_statusRepository)  GetKyc_statusByStatusCode(ctx context.Context, statuscode string) (kyc_status *model.Kyc_status, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Kyc_statusRepository -> GetKyc_statusByStatusCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetKyc_statusByStatusCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetKyc_statusByStatusCode.statuscode", statuscode)

	kyc_status = new(model.Kyc_status)
	row := t.PGRead.QueryRow(ctx, SQL_GET_KYC_STATUS_BY_STATUS_CODE, statuscode)
		err = row.Scan(
			&kyc_status.ID,
			&kyc_status.StatusCode,
			&kyc_status.Name,
			&kyc_status.Description,
			&kyc_status.AllowsTransactions,
			&kyc_status.MaxTransactionAmount,
			&kyc_status.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Kyc_statusRepository.repository.GetKyc_statusBystatuscode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return kyc_status, nil
}
func (t Kyc_statusRepository)  DeleteKyc_statusById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Kyc_statusRepository -> DeleteKyc_statusById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteKyc_statusById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_KYC_STATUS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Kyc_statusRepository.repository.DeleteKyc_statusById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteKyc_statusById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteKyc_statusById.deleted", result)
	return true, err
}
func (t Kyc_statusRepository)  InsertKyc_status(ctx context.Context,kyc_status *model.Kyc_status) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Kyc_statusRepository -> InsertKyc_status", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertKyc_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertKyc_status.statuscode", kyc_status.StatusCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_KYC_STATUS_INSERT,
			kyc_status.StatusCode,
			kyc_status.Name,
			kyc_status.Description,
			kyc_status.AllowsTransactions,
			kyc_status.MaxTransactionAmount,
			kyc_status.CreatedAt,
	).Scan(&kyc_status.ID)

	if err != nil {
		t.log.Error(ctx, "Kyc_statusRepository.repository.InsertKyc_status.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertKyc_status.inserted_id", kyc_status.ID)
   return kyc_status.ID, nil

}
func (t Kyc_statusRepository)  UpdateKyc_status(ctx context.Context,kyc_status *model.Kyc_status, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Kyc_statusRepository -> UpdateKyc_status", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateKyc_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateKyc_status.id", id)
	tracker.AddParam("repository.UpdateKyc_status.statuscode", kyc_status.StatusCode)

	kyc_status.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_KYC_STATUS_UPDATE, 
			kyc_status.StatusCode,
			kyc_status.Name,
			kyc_status.Description,
			kyc_status.AllowsTransactions,
			kyc_status.MaxTransactionAmount,
			kyc_status.CreatedAt,
			kyc_status.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Kyc_statusRepository.repository.UpdateKyc_status.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateKyc_status.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateKyc_status.rows_affected", rowsAffected)
	return nil
}

