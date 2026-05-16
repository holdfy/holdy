package document_statusRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Document_statusRepositoryIF interface {
     GetDocument_status(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetDocument_statusById(ctx context.Context, id int64) (*model.Document_status, error)
     GetDocument_statusByStatusCode(ctx context.Context, statuscode string) (*model.Document_status, error)
     InsertDocument_status(ctx context.Context, document_status *model.Document_status) (int64, error)
     UpdateDocument_status(ctx context.Context, document_status *model.Document_status, id int64) error
     DeleteDocument_statusById(ctx context.Context, id int64) (bool, error)
}
 type Document_statusRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewDocument_statusRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Document_statusRepository{
    return &Document_statusRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Document_status"),
     }
}
func (t Document_statusRepository)  GetDocument_status(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Document_statusRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetDocument_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetDocument_status.offset", offset)
	tracker.AddParam("repository.GetDocument_status.limit", limit)
	itemsPage 			= model.ItemsPage{}
	document_statuss := []model.Document_status{}

	rows, err := t.PGRead.Query(ctx, SQL_DOCUMENT_STATUS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Document_statusRepository.repository.GetDocument_statuss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var document_status model.Document_status
		err := rows.Scan(
			&document_status.ID,
			&document_status.StatusCode,
			&document_status.Name,
			&document_status.Description,
			&document_status.IsFinal,
			&document_status.RequiresAction,
			&document_status.NextPossibleStatus,
			&document_status.IsActive,
			&document_status.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Document_statusRepository.repository.GetDocument_statuss.Scan: ", err.Error())
			return itemsPage, err
		}
		document_statuss = append(document_statuss, document_status)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Document_statusRepository.repository.GetDocument_statuss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(document_statuss) > 0 {
		qtyRecords = document_statuss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = document_statuss

	tracker.AddResult("repository.GetDocument_status.rows_returned", len(document_statuss))
	tracker.AddResult("repository.GetDocument_status.total_count", len(document_statuss))

	return itemsPage, nil
}
func (t Document_statusRepository)  GetDocument_statusById(ctx context.Context, id int64) (document_status *model.Document_status, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Document_statusRepository -> GetDocument_statusById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetDocument_statusById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetDocument_statusById.id", id)

	document_status = new(model.Document_status)
	row := t.PGRead.QueryRow(ctx, SQL_GET_DOCUMENT_STATUS_BY_ID, id)
		err = row.Scan(
			&document_status.ID,
			&document_status.StatusCode,
			&document_status.Name,
			&document_status.Description,
			&document_status.IsFinal,
			&document_status.RequiresAction,
			&document_status.NextPossibleStatus,
			&document_status.IsActive,
			&document_status.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Document_statusRepository.repository.GetDocument_statusById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetDocument_statusById.found", true)
	return document_status, nil
}
func (t Document_statusRepository)  GetDocument_statusByStatusCode(ctx context.Context, statuscode string) (document_status *model.Document_status, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Document_statusRepository -> GetDocument_statusByStatusCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetDocument_statusByStatusCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetDocument_statusByStatusCode.statuscode", statuscode)

	document_status = new(model.Document_status)
	row := t.PGRead.QueryRow(ctx, SQL_GET_DOCUMENT_STATUS_BY_STATUS_CODE, statuscode)
		err = row.Scan(
			&document_status.ID,
			&document_status.StatusCode,
			&document_status.Name,
			&document_status.Description,
			&document_status.IsFinal,
			&document_status.RequiresAction,
			&document_status.NextPossibleStatus,
			&document_status.IsActive,
			&document_status.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Document_statusRepository.repository.GetDocument_statusBystatuscode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return document_status, nil
}
func (t Document_statusRepository)  DeleteDocument_statusById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Document_statusRepository -> DeleteDocument_statusById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteDocument_statusById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_DOCUMENT_STATUS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Document_statusRepository.repository.DeleteDocument_statusById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteDocument_statusById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteDocument_statusById.deleted", result)
	return true, err
}
func (t Document_statusRepository)  InsertDocument_status(ctx context.Context,document_status *model.Document_status) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Document_statusRepository -> InsertDocument_status", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertDocument_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertDocument_status.statuscode", document_status.StatusCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_DOCUMENT_STATUS_INSERT,
			document_status.StatusCode,
			document_status.Name,
			document_status.Description,
			document_status.IsFinal,
			document_status.RequiresAction,
			document_status.NextPossibleStatus,
			document_status.IsActive,
			document_status.CreatedAt,
	).Scan(&document_status.ID)

	if err != nil {
		t.log.Error(ctx, "Document_statusRepository.repository.InsertDocument_status.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertDocument_status.inserted_id", document_status.ID)
   return document_status.ID, nil

}
func (t Document_statusRepository)  UpdateDocument_status(ctx context.Context,document_status *model.Document_status, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Document_statusRepository -> UpdateDocument_status", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateDocument_status")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateDocument_status.id", id)
	tracker.AddParam("repository.UpdateDocument_status.statuscode", document_status.StatusCode)

	document_status.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_DOCUMENT_STATUS_UPDATE, 
			document_status.StatusCode,
			document_status.Name,
			document_status.Description,
			document_status.IsFinal,
			document_status.RequiresAction,
			document_status.NextPossibleStatus,
			document_status.IsActive,
			document_status.CreatedAt,
			document_status.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Document_statusRepository.repository.UpdateDocument_status.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateDocument_status.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateDocument_status.rows_affected", rowsAffected)
	return nil
}

