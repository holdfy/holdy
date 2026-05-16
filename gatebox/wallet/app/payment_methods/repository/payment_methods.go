package payment_methodsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Payment_methodsRepositoryIF interface {
     GetPayment_methods(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetPayment_methodsById(ctx context.Context, id int64) (*model.Payment_methods, error)
     GetPayment_methodsByMethodCode(ctx context.Context, methodcode string) (*model.Payment_methods, error)
     InsertPayment_methods(ctx context.Context, payment_methods *model.Payment_methods) (int64, error)
     UpdatePayment_methods(ctx context.Context, payment_methods *model.Payment_methods, id int64) error
     DeletePayment_methodsById(ctx context.Context, id int64) (bool, error)
}
 type Payment_methodsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewPayment_methodsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Payment_methodsRepository{
    return &Payment_methodsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Payment_methods"),
     }
}
func (t Payment_methodsRepository)  GetPayment_methods(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Payment_methodsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetPayment_methods")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetPayment_methods.offset", offset)
	tracker.AddParam("repository.GetPayment_methods.limit", limit)
	itemsPage 			= model.ItemsPage{}
	payment_methodss := []model.Payment_methods{}

	rows, err := t.PGRead.Query(ctx, SQL_PAYMENT_METHODS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Payment_methodsRepository.repository.GetPayment_methodss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var payment_methods model.Payment_methods
		err := rows.Scan(
			&payment_methods.ID,
			&payment_methods.MethodCode,
			&payment_methods.Name,
			&payment_methods.Description,
			&payment_methods.RequiresExternalAuth,
			&payment_methods.ProcessingTimeMinutes,
			&payment_methods.IsActive,
			&payment_methods.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Payment_methodsRepository.repository.GetPayment_methodss.Scan: ", err.Error())
			return itemsPage, err
		}
		payment_methodss = append(payment_methodss, payment_methods)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Payment_methodsRepository.repository.GetPayment_methodss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(payment_methodss) > 0 {
		qtyRecords = payment_methodss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = payment_methodss

	tracker.AddResult("repository.GetPayment_methods.rows_returned", len(payment_methodss))
	tracker.AddResult("repository.GetPayment_methods.total_count", len(payment_methodss))

	return itemsPage, nil
}
func (t Payment_methodsRepository)  GetPayment_methodsById(ctx context.Context, id int64) (payment_methods *model.Payment_methods, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Payment_methodsRepository -> GetPayment_methodsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetPayment_methodsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetPayment_methodsById.id", id)

	payment_methods = new(model.Payment_methods)
	row := t.PGRead.QueryRow(ctx, SQL_GET_PAYMENT_METHODS_BY_ID, id)
		err = row.Scan(
			&payment_methods.ID,
			&payment_methods.MethodCode,
			&payment_methods.Name,
			&payment_methods.Description,
			&payment_methods.RequiresExternalAuth,
			&payment_methods.ProcessingTimeMinutes,
			&payment_methods.IsActive,
			&payment_methods.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Payment_methodsRepository.repository.GetPayment_methodsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetPayment_methodsById.found", true)
	return payment_methods, nil
}
func (t Payment_methodsRepository)  GetPayment_methodsByMethodCode(ctx context.Context, methodcode string) (payment_methods *model.Payment_methods, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Payment_methodsRepository -> GetPayment_methodsByMethodCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetPayment_methodsByMethodCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetPayment_methodsByMethodCode.methodcode", methodcode)

	payment_methods = new(model.Payment_methods)
	row := t.PGRead.QueryRow(ctx, SQL_GET_PAYMENT_METHODS_BY_METHOD_CODE, methodcode)
		err = row.Scan(
			&payment_methods.ID,
			&payment_methods.MethodCode,
			&payment_methods.Name,
			&payment_methods.Description,
			&payment_methods.RequiresExternalAuth,
			&payment_methods.ProcessingTimeMinutes,
			&payment_methods.IsActive,
			&payment_methods.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Payment_methodsRepository.repository.GetPayment_methodsBymethodcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return payment_methods, nil
}
func (t Payment_methodsRepository)  DeletePayment_methodsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Payment_methodsRepository -> DeletePayment_methodsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeletePayment_methodsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_PAYMENT_METHODS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Payment_methodsRepository.repository.DeletePayment_methodsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeletePayment_methodsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeletePayment_methodsById.deleted", result)
	return true, err
}
func (t Payment_methodsRepository)  InsertPayment_methods(ctx context.Context,payment_methods *model.Payment_methods) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Payment_methodsRepository -> InsertPayment_methods", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertPayment_methods")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertPayment_methods.methodcode", payment_methods.MethodCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_PAYMENT_METHODS_INSERT,
			payment_methods.MethodCode,
			payment_methods.Name,
			payment_methods.Description,
			payment_methods.RequiresExternalAuth,
			payment_methods.ProcessingTimeMinutes,
			payment_methods.IsActive,
			payment_methods.CreatedAt,
	).Scan(&payment_methods.ID)

	if err != nil {
		t.log.Error(ctx, "Payment_methodsRepository.repository.InsertPayment_methods.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertPayment_methods.inserted_id", payment_methods.ID)
   return payment_methods.ID, nil

}
func (t Payment_methodsRepository)  UpdatePayment_methods(ctx context.Context,payment_methods *model.Payment_methods, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Payment_methodsRepository -> UpdatePayment_methods", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdatePayment_methods")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdatePayment_methods.id", id)
	tracker.AddParam("repository.UpdatePayment_methods.methodcode", payment_methods.MethodCode)

	payment_methods.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_PAYMENT_METHODS_UPDATE, 
			payment_methods.MethodCode,
			payment_methods.Name,
			payment_methods.Description,
			payment_methods.RequiresExternalAuth,
			payment_methods.ProcessingTimeMinutes,
			payment_methods.IsActive,
			payment_methods.CreatedAt,
			payment_methods.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Payment_methodsRepository.repository.UpdatePayment_methods.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdatePayment_methods.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdatePayment_methods.rows_affected", rowsAffected)
	return nil
}

