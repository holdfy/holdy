package restriction_typesRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Restriction_typesRepositoryIF interface {
     GetRestriction_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetRestriction_typesById(ctx context.Context, id int64) (*model.Restriction_types, error)
     GetRestriction_typesByTypeCode(ctx context.Context, typecode string) (*model.Restriction_types, error)
     InsertRestriction_types(ctx context.Context, restriction_types *model.Restriction_types) (int64, error)
     UpdateRestriction_types(ctx context.Context, restriction_types *model.Restriction_types, id int64) error
     DeleteRestriction_typesById(ctx context.Context, id int64) (bool, error)
}
 type Restriction_typesRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewRestriction_typesRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Restriction_typesRepository{
    return &Restriction_typesRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Restriction_types"),
     }
}
func (t Restriction_typesRepository)  GetRestriction_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Restriction_typesRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetRestriction_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetRestriction_types.offset", offset)
	tracker.AddParam("repository.GetRestriction_types.limit", limit)
	itemsPage 			= model.ItemsPage{}
	restriction_typess := []model.Restriction_types{}

	rows, err := t.PGRead.Query(ctx, SQL_RESTRICTION_TYPES_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Restriction_typesRepository.repository.GetRestriction_typess.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var restriction_types model.Restriction_types
		err := rows.Scan(
			&restriction_types.ID,
			&restriction_types.TypeCode,
			&restriction_types.Name,
			&restriction_types.Description,
			&restriction_types.AffectsTransactions,
			&restriction_types.AffectsLogin,
			&restriction_types.AffectsBiometric,
			&restriction_types.CanAutoExpire,
			&restriction_types.DefaultDurationHours,
			&restriction_types.SeverityLevel,
			&restriction_types.IsActive,
			&restriction_types.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Restriction_typesRepository.repository.GetRestriction_typess.Scan: ", err.Error())
			return itemsPage, err
		}
		restriction_typess = append(restriction_typess, restriction_types)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Restriction_typesRepository.repository.GetRestriction_typess.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(restriction_typess) > 0 {
		qtyRecords = restriction_typess[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = restriction_typess

	tracker.AddResult("repository.GetRestriction_types.rows_returned", len(restriction_typess))
	tracker.AddResult("repository.GetRestriction_types.total_count", len(restriction_typess))

	return itemsPage, nil
}
func (t Restriction_typesRepository)  GetRestriction_typesById(ctx context.Context, id int64) (restriction_types *model.Restriction_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Restriction_typesRepository -> GetRestriction_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetRestriction_typesById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetRestriction_typesById.id", id)

	restriction_types = new(model.Restriction_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_RESTRICTION_TYPES_BY_ID, id)
		err = row.Scan(
			&restriction_types.ID,
			&restriction_types.TypeCode,
			&restriction_types.Name,
			&restriction_types.Description,
			&restriction_types.AffectsTransactions,
			&restriction_types.AffectsLogin,
			&restriction_types.AffectsBiometric,
			&restriction_types.CanAutoExpire,
			&restriction_types.DefaultDurationHours,
			&restriction_types.SeverityLevel,
			&restriction_types.IsActive,
			&restriction_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Restriction_typesRepository.repository.GetRestriction_typesById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetRestriction_typesById.found", true)
	return restriction_types, nil
}
func (t Restriction_typesRepository)  GetRestriction_typesByTypeCode(ctx context.Context, typecode string) (restriction_types *model.Restriction_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Restriction_typesRepository -> GetRestriction_typesByTypeCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetRestriction_typesByTypeCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetRestriction_typesByTypeCode.typecode", typecode)

	restriction_types = new(model.Restriction_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_RESTRICTION_TYPES_BY_TYPE_CODE, typecode)
		err = row.Scan(
			&restriction_types.ID,
			&restriction_types.TypeCode,
			&restriction_types.Name,
			&restriction_types.Description,
			&restriction_types.AffectsTransactions,
			&restriction_types.AffectsLogin,
			&restriction_types.AffectsBiometric,
			&restriction_types.CanAutoExpire,
			&restriction_types.DefaultDurationHours,
			&restriction_types.SeverityLevel,
			&restriction_types.IsActive,
			&restriction_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Restriction_typesRepository.repository.GetRestriction_typesBytypecode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return restriction_types, nil
}
func (t Restriction_typesRepository)  DeleteRestriction_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Restriction_typesRepository -> DeleteRestriction_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteRestriction_typesById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_RESTRICTION_TYPES_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Restriction_typesRepository.repository.DeleteRestriction_typesById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteRestriction_typesById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteRestriction_typesById.deleted", result)
	return true, err
}
func (t Restriction_typesRepository)  InsertRestriction_types(ctx context.Context,restriction_types *model.Restriction_types) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Restriction_typesRepository -> InsertRestriction_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertRestriction_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertRestriction_types.typecode", restriction_types.TypeCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_RESTRICTION_TYPES_INSERT,
			restriction_types.TypeCode,
			restriction_types.Name,
			restriction_types.Description,
			restriction_types.AffectsTransactions,
			restriction_types.AffectsLogin,
			restriction_types.AffectsBiometric,
			restriction_types.CanAutoExpire,
			restriction_types.DefaultDurationHours,
			restriction_types.SeverityLevel,
			restriction_types.IsActive,
			restriction_types.CreatedAt,
	).Scan(&restriction_types.ID)

	if err != nil {
		t.log.Error(ctx, "Restriction_typesRepository.repository.InsertRestriction_types.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertRestriction_types.inserted_id", restriction_types.ID)
   return restriction_types.ID, nil

}
func (t Restriction_typesRepository)  UpdateRestriction_types(ctx context.Context,restriction_types *model.Restriction_types, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Restriction_typesRepository -> UpdateRestriction_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateRestriction_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateRestriction_types.id", id)
	tracker.AddParam("repository.UpdateRestriction_types.typecode", restriction_types.TypeCode)

	restriction_types.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_RESTRICTION_TYPES_UPDATE, 
			restriction_types.TypeCode,
			restriction_types.Name,
			restriction_types.Description,
			restriction_types.AffectsTransactions,
			restriction_types.AffectsLogin,
			restriction_types.AffectsBiometric,
			restriction_types.CanAutoExpire,
			restriction_types.DefaultDurationHours,
			restriction_types.SeverityLevel,
			restriction_types.IsActive,
			restriction_types.CreatedAt,
			restriction_types.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Restriction_typesRepository.repository.UpdateRestriction_types.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateRestriction_types.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateRestriction_types.rows_affected", rowsAffected)
	return nil
}

