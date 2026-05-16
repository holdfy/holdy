package user_documentsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type User_documentsRepositoryIF interface {
     GetUser_documents(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetUser_documentsById(ctx context.Context, id int64) (*model.User_documents, error)
     GetUser_documentsByDocumentCode(ctx context.Context, documentcode string) (*model.User_documents, error)
     InsertUser_documents(ctx context.Context, user_documents *model.User_documents) (int64, error)
     UpdateUser_documents(ctx context.Context, user_documents *model.User_documents, id int64) error
     DeleteUser_documentsById(ctx context.Context, id int64) (bool, error)
}
 type User_documentsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewUser_documentsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *User_documentsRepository{
    return &User_documentsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("User_documents"),
     }
}
func (t User_documentsRepository)  GetUser_documents(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_documentsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_documents")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_documents.offset", offset)
	tracker.AddParam("repository.GetUser_documents.limit", limit)
	itemsPage 			= model.ItemsPage{}
	user_documentss := []model.User_documents{}

	rows, err := t.PGRead.Query(ctx, SQL_USER_DOCUMENTS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "User_documentsRepository.repository.GetUser_documentss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var user_documents model.User_documents
		err := rows.Scan(
			&user_documents.ID,
			&user_documents.DocumentCode,
			&user_documents.UserId,
			&user_documents.IdDocumentType,
			&user_documents.DocumentNumber,
			&user_documents.FilePath,
			&user_documents.FileHash,
			&user_documents.IdStatusDocuments,
			&user_documents.VerifiedAt,
			&user_documents.VerifiedBy,
			&user_documents.RejectionReason,
			&user_documents.Metadata,
			&user_documents.CreatedAt,
			&user_documents.UpdatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "User_documentsRepository.repository.GetUser_documentss.Scan: ", err.Error())
			return itemsPage, err
		}
		user_documentss = append(user_documentss, user_documents)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "User_documentsRepository.repository.GetUser_documentss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(user_documentss) > 0 {
		qtyRecords = user_documentss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = user_documentss

	tracker.AddResult("repository.GetUser_documents.rows_returned", len(user_documentss))
	tracker.AddResult("repository.GetUser_documents.total_count", len(user_documentss))

	return itemsPage, nil
}
func (t User_documentsRepository)  GetUser_documentsById(ctx context.Context, id int64) (user_documents *model.User_documents, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_documentsRepository -> GetUser_documentsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_documentsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_documentsById.id", id)

	user_documents = new(model.User_documents)
	row := t.PGRead.QueryRow(ctx, SQL_GET_USER_DOCUMENTS_BY_ID, id)
		err = row.Scan(
			&user_documents.ID,
			&user_documents.DocumentCode,
			&user_documents.UserId,
			&user_documents.IdDocumentType,
			&user_documents.DocumentNumber,
			&user_documents.FilePath,
			&user_documents.FileHash,
			&user_documents.IdStatusDocuments,
			&user_documents.VerifiedAt,
			&user_documents.VerifiedBy,
			&user_documents.RejectionReason,
			&user_documents.Metadata,
			&user_documents.CreatedAt,
			&user_documents.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"User_documentsRepository.repository.GetUser_documentsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetUser_documentsById.found", true)
	return user_documents, nil
}
func (t User_documentsRepository)  GetUser_documentsByDocumentCode(ctx context.Context, documentcode string) (user_documents *model.User_documents, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_documentsRepository -> GetUser_documentsByDocumentCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetUser_documentsByDocumentCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetUser_documentsByDocumentCode.documentcode", documentcode)

	user_documents = new(model.User_documents)
	row := t.PGRead.QueryRow(ctx, SQL_GET_USER_DOCUMENTS_BY_DOCUMENT_CODE, documentcode)
		err = row.Scan(
			&user_documents.ID,
			&user_documents.DocumentCode,
			&user_documents.UserId,
			&user_documents.IdDocumentType,
			&user_documents.DocumentNumber,
			&user_documents.FilePath,
			&user_documents.FileHash,
			&user_documents.IdStatusDocuments,
			&user_documents.VerifiedAt,
			&user_documents.VerifiedBy,
			&user_documents.RejectionReason,
			&user_documents.Metadata,
			&user_documents.CreatedAt,
			&user_documents.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"User_documentsRepository.repository.GetUser_documentsBydocumentcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return user_documents, nil
}
func (t User_documentsRepository)  DeleteUser_documentsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_documentsRepository -> DeleteUser_documentsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteUser_documentsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_USER_DOCUMENTS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"User_documentsRepository.repository.DeleteUser_documentsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteUser_documentsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteUser_documentsById.deleted", result)
	return true, err
}
func (t User_documentsRepository)  InsertUser_documents(ctx context.Context,user_documents *model.User_documents) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_documentsRepository -> InsertUser_documents", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertUser_documents")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertUser_documents.documentcode", user_documents.DocumentCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_USER_DOCUMENTS_INSERT,
			user_documents.DocumentCode,
			user_documents.UserId,
			user_documents.IdDocumentType,
			user_documents.DocumentNumber,
			user_documents.FilePath,
			user_documents.FileHash,
			user_documents.IdStatusDocuments,
			user_documents.VerifiedAt,
			user_documents.VerifiedBy,
			user_documents.RejectionReason,
			user_documents.Metadata,
			user_documents.CreatedAt,
			user_documents.UpdatedAt,
	).Scan(&user_documents.ID)

	if err != nil {
		t.log.Error(ctx, "User_documentsRepository.repository.InsertUser_documents.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertUser_documents.inserted_id", user_documents.ID)
   return user_documents.ID, nil

}
func (t User_documentsRepository)  UpdateUser_documents(ctx context.Context,user_documents *model.User_documents, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("User_documentsRepository -> UpdateUser_documents", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateUser_documents")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateUser_documents.id", id)
	tracker.AddParam("repository.UpdateUser_documents.documentcode", user_documents.DocumentCode)

	user_documents.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_USER_DOCUMENTS_UPDATE, 
			user_documents.DocumentCode,
			user_documents.UserId,
			user_documents.IdDocumentType,
			user_documents.DocumentNumber,
			user_documents.FilePath,
			user_documents.FileHash,
			user_documents.IdStatusDocuments,
			user_documents.VerifiedAt,
			user_documents.VerifiedBy,
			user_documents.RejectionReason,
			user_documents.Metadata,
			user_documents.CreatedAt,
			user_documents.UpdatedAt,
			user_documents.ID,
   )
	if err != nil {
		t.log.Error(ctx, "User_documentsRepository.repository.UpdateUser_documents.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateUser_documents.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateUser_documents.rows_affected", rowsAffected)
	return nil
}

