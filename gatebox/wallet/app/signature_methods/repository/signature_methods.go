package signature_methodsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Signature_methodsRepositoryIF interface {
     GetSignature_methods(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetSignature_methodsById(ctx context.Context, id int64) (*model.Signature_methods, error)
     GetSignature_methodsByMethodCode(ctx context.Context, methodcode string) (*model.Signature_methods, error)
     InsertSignature_methods(ctx context.Context, signature_methods *model.Signature_methods) (int64, error)
     UpdateSignature_methods(ctx context.Context, signature_methods *model.Signature_methods, id int64) error
     DeleteSignature_methodsById(ctx context.Context, id int64) (bool, error)
}
 type Signature_methodsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewSignature_methodsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Signature_methodsRepository{
    return &Signature_methodsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Signature_methods"),
     }
}
func (t Signature_methodsRepository)  GetSignature_methods(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Signature_methodsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSignature_methods")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSignature_methods.offset", offset)
	tracker.AddParam("repository.GetSignature_methods.limit", limit)
	itemsPage 			= model.ItemsPage{}
	signature_methodss := []model.Signature_methods{}

	rows, err := t.PGRead.Query(ctx, SQL_SIGNATURE_METHODS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Signature_methodsRepository.repository.GetSignature_methodss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var signature_methods model.Signature_methods
		err := rows.Scan(
			&signature_methods.ID,
			&signature_methods.MethodCode,
			&signature_methods.Name,
			&signature_methods.Description,
			&signature_methods.SecurityLevel,
			&signature_methods.IsActive,
			&signature_methods.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Signature_methodsRepository.repository.GetSignature_methodss.Scan: ", err.Error())
			return itemsPage, err
		}
		signature_methodss = append(signature_methodss, signature_methods)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Signature_methodsRepository.repository.GetSignature_methodss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(signature_methodss) > 0 {
		qtyRecords = signature_methodss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = signature_methodss

	tracker.AddResult("repository.GetSignature_methods.rows_returned", len(signature_methodss))
	tracker.AddResult("repository.GetSignature_methods.total_count", len(signature_methodss))

	return itemsPage, nil
}
func (t Signature_methodsRepository)  GetSignature_methodsById(ctx context.Context, id int64) (signature_methods *model.Signature_methods, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Signature_methodsRepository -> GetSignature_methodsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSignature_methodsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSignature_methodsById.id", id)

	signature_methods = new(model.Signature_methods)
	row := t.PGRead.QueryRow(ctx, SQL_GET_SIGNATURE_METHODS_BY_ID, id)
		err = row.Scan(
			&signature_methods.ID,
			&signature_methods.MethodCode,
			&signature_methods.Name,
			&signature_methods.Description,
			&signature_methods.SecurityLevel,
			&signature_methods.IsActive,
			&signature_methods.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Signature_methodsRepository.repository.GetSignature_methodsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetSignature_methodsById.found", true)
	return signature_methods, nil
}
func (t Signature_methodsRepository)  GetSignature_methodsByMethodCode(ctx context.Context, methodcode string) (signature_methods *model.Signature_methods, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Signature_methodsRepository -> GetSignature_methodsByMethodCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSignature_methodsByMethodCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSignature_methodsByMethodCode.methodcode", methodcode)

	signature_methods = new(model.Signature_methods)
	row := t.PGRead.QueryRow(ctx, SQL_GET_SIGNATURE_METHODS_BY_METHOD_CODE, methodcode)
		err = row.Scan(
			&signature_methods.ID,
			&signature_methods.MethodCode,
			&signature_methods.Name,
			&signature_methods.Description,
			&signature_methods.SecurityLevel,
			&signature_methods.IsActive,
			&signature_methods.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Signature_methodsRepository.repository.GetSignature_methodsBymethodcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return signature_methods, nil
}
func (t Signature_methodsRepository)  DeleteSignature_methodsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Signature_methodsRepository -> DeleteSignature_methodsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteSignature_methodsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_SIGNATURE_METHODS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Signature_methodsRepository.repository.DeleteSignature_methodsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteSignature_methodsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteSignature_methodsById.deleted", result)
	return true, err
}
func (t Signature_methodsRepository)  InsertSignature_methods(ctx context.Context,signature_methods *model.Signature_methods) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Signature_methodsRepository -> InsertSignature_methods", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertSignature_methods")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertSignature_methods.methodcode", signature_methods.MethodCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_SIGNATURE_METHODS_INSERT,
			signature_methods.MethodCode,
			signature_methods.Name,
			signature_methods.Description,
			signature_methods.SecurityLevel,
			signature_methods.IsActive,
			signature_methods.CreatedAt,
	).Scan(&signature_methods.ID)

	if err != nil {
		t.log.Error(ctx, "Signature_methodsRepository.repository.InsertSignature_methods.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertSignature_methods.inserted_id", signature_methods.ID)
   return signature_methods.ID, nil

}
func (t Signature_methodsRepository)  UpdateSignature_methods(ctx context.Context,signature_methods *model.Signature_methods, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Signature_methodsRepository -> UpdateSignature_methods", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateSignature_methods")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateSignature_methods.id", id)
	tracker.AddParam("repository.UpdateSignature_methods.methodcode", signature_methods.MethodCode)

	signature_methods.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_SIGNATURE_METHODS_UPDATE, 
			signature_methods.MethodCode,
			signature_methods.Name,
			signature_methods.Description,
			signature_methods.SecurityLevel,
			signature_methods.IsActive,
			signature_methods.CreatedAt,
			signature_methods.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Signature_methodsRepository.repository.UpdateSignature_methods.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateSignature_methods.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateSignature_methods.rows_affected", rowsAffected)
	return nil
}

