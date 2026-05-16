package document_typesRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Document_typesRepositoryIF interface {
     GetDocument_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetDocument_typesById(ctx context.Context, id int64) (*model.Document_types, error)
     GetDocument_typesByTypeCode(ctx context.Context, typecode string) (*model.Document_types, error)
     InsertDocument_types(ctx context.Context, document_types *model.Document_types) (int64, error)
     UpdateDocument_types(ctx context.Context, document_types *model.Document_types, id int64) error
     DeleteDocument_typesById(ctx context.Context, id int64) (bool, error)
}
 type Document_typesRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewDocument_typesRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Document_typesRepository{
    return &Document_typesRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Document_types"),
     }
}
func (t Document_typesRepository)  GetDocument_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Document_typesRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetDocument_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetDocument_types.offset", offset)
	tracker.AddParam("repository.GetDocument_types.limit", limit)
	itemsPage 			= model.ItemsPage{}
	document_typess := []model.Document_types{}

	rows, err := t.PGRead.Query(ctx, SQL_DOCUMENT_TYPES_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Document_typesRepository.repository.GetDocument_typess.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var document_types model.Document_types
		err := rows.Scan(
			&document_types.ID,
			&document_types.TypeCode,
			&document_types.Name,
			&document_types.Description,
			&document_types.IsRequired,
			&document_types.MaxFileSizeMb,
			&document_types.AllowedExtensions,
			&document_types.IsActive,
			&document_types.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Document_typesRepository.repository.GetDocument_typess.Scan: ", err.Error())
			return itemsPage, err
		}
		document_typess = append(document_typess, document_types)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Document_typesRepository.repository.GetDocument_typess.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(document_typess) > 0 {
		qtyRecords = document_typess[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = document_typess

	tracker.AddResult("repository.GetDocument_types.rows_returned", len(document_typess))
	tracker.AddResult("repository.GetDocument_types.total_count", len(document_typess))

	return itemsPage, nil
}
func (t Document_typesRepository)  GetDocument_typesById(ctx context.Context, id int64) (document_types *model.Document_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Document_typesRepository -> GetDocument_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetDocument_typesById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetDocument_typesById.id", id)

	document_types = new(model.Document_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_DOCUMENT_TYPES_BY_ID, id)
		err = row.Scan(
			&document_types.ID,
			&document_types.TypeCode,
			&document_types.Name,
			&document_types.Description,
			&document_types.IsRequired,
			&document_types.MaxFileSizeMb,
			&document_types.AllowedExtensions,
			&document_types.IsActive,
			&document_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Document_typesRepository.repository.GetDocument_typesById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetDocument_typesById.found", true)
	return document_types, nil
}
func (t Document_typesRepository)  GetDocument_typesByTypeCode(ctx context.Context, typecode string) (document_types *model.Document_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Document_typesRepository -> GetDocument_typesByTypeCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetDocument_typesByTypeCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetDocument_typesByTypeCode.typecode", typecode)

	document_types = new(model.Document_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_DOCUMENT_TYPES_BY_TYPE_CODE, typecode)
		err = row.Scan(
			&document_types.ID,
			&document_types.TypeCode,
			&document_types.Name,
			&document_types.Description,
			&document_types.IsRequired,
			&document_types.MaxFileSizeMb,
			&document_types.AllowedExtensions,
			&document_types.IsActive,
			&document_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Document_typesRepository.repository.GetDocument_typesBytypecode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return document_types, nil
}
func (t Document_typesRepository)  DeleteDocument_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Document_typesRepository -> DeleteDocument_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteDocument_typesById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_DOCUMENT_TYPES_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Document_typesRepository.repository.DeleteDocument_typesById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteDocument_typesById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteDocument_typesById.deleted", result)
	return true, err
}
func (t Document_typesRepository)  InsertDocument_types(ctx context.Context,document_types *model.Document_types) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Document_typesRepository -> InsertDocument_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertDocument_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertDocument_types.typecode", document_types.TypeCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_DOCUMENT_TYPES_INSERT,
			document_types.TypeCode,
			document_types.Name,
			document_types.Description,
			document_types.IsRequired,
			document_types.MaxFileSizeMb,
			document_types.AllowedExtensions,
			document_types.IsActive,
			document_types.CreatedAt,
	).Scan(&document_types.ID)

	if err != nil {
		t.log.Error(ctx, "Document_typesRepository.repository.InsertDocument_types.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertDocument_types.inserted_id", document_types.ID)
   return document_types.ID, nil

}
func (t Document_typesRepository)  UpdateDocument_types(ctx context.Context,document_types *model.Document_types, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Document_typesRepository -> UpdateDocument_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateDocument_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateDocument_types.id", id)
	tracker.AddParam("repository.UpdateDocument_types.typecode", document_types.TypeCode)

	document_types.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_DOCUMENT_TYPES_UPDATE, 
			document_types.TypeCode,
			document_types.Name,
			document_types.Description,
			document_types.IsRequired,
			document_types.MaxFileSizeMb,
			document_types.AllowedExtensions,
			document_types.IsActive,
			document_types.CreatedAt,
			document_types.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Document_typesRepository.repository.UpdateDocument_types.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateDocument_types.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateDocument_types.rows_affected", rowsAffected)
	return nil
}

