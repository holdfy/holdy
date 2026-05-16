package hand_typesRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Hand_typesRepositoryIF interface {
     GetHand_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetHand_typesById(ctx context.Context, id int64) (*model.Hand_types, error)
     GetHand_typesByTypeCode(ctx context.Context, typecode string) (*model.Hand_types, error)
     InsertHand_types(ctx context.Context, hand_types *model.Hand_types) (int64, error)
     UpdateHand_types(ctx context.Context, hand_types *model.Hand_types, id int64) error
     DeleteHand_typesById(ctx context.Context, id int64) (bool, error)
}
 type Hand_typesRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewHand_typesRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Hand_typesRepository{
    return &Hand_typesRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Hand_types"),
     }
}
func (t Hand_typesRepository)  GetHand_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Hand_typesRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetHand_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetHand_types.offset", offset)
	tracker.AddParam("repository.GetHand_types.limit", limit)
	itemsPage 			= model.ItemsPage{}
	hand_typess := []model.Hand_types{}

	rows, err := t.PGRead.Query(ctx, SQL_HAND_TYPES_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Hand_typesRepository.repository.GetHand_typess.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var hand_types model.Hand_types
		err := rows.Scan(
			&hand_types.ID,
			&hand_types.TypeCode,
			&hand_types.Name,
			&hand_types.Description,
			&hand_types.IsActive,
			&hand_types.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Hand_typesRepository.repository.GetHand_typess.Scan: ", err.Error())
			return itemsPage, err
		}
		hand_typess = append(hand_typess, hand_types)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Hand_typesRepository.repository.GetHand_typess.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(hand_typess) > 0 {
		qtyRecords = hand_typess[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = hand_typess

	tracker.AddResult("repository.GetHand_types.rows_returned", len(hand_typess))
	tracker.AddResult("repository.GetHand_types.total_count", len(hand_typess))

	return itemsPage, nil
}
func (t Hand_typesRepository)  GetHand_typesById(ctx context.Context, id int64) (hand_types *model.Hand_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Hand_typesRepository -> GetHand_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetHand_typesById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetHand_typesById.id", id)

	hand_types = new(model.Hand_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_HAND_TYPES_BY_ID, id)
		err = row.Scan(
			&hand_types.ID,
			&hand_types.TypeCode,
			&hand_types.Name,
			&hand_types.Description,
			&hand_types.IsActive,
			&hand_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Hand_typesRepository.repository.GetHand_typesById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetHand_typesById.found", true)
	return hand_types, nil
}
func (t Hand_typesRepository)  GetHand_typesByTypeCode(ctx context.Context, typecode string) (hand_types *model.Hand_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Hand_typesRepository -> GetHand_typesByTypeCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetHand_typesByTypeCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetHand_typesByTypeCode.typecode", typecode)

	hand_types = new(model.Hand_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_HAND_TYPES_BY_TYPE_CODE, typecode)
		err = row.Scan(
			&hand_types.ID,
			&hand_types.TypeCode,
			&hand_types.Name,
			&hand_types.Description,
			&hand_types.IsActive,
			&hand_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Hand_typesRepository.repository.GetHand_typesBytypecode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return hand_types, nil
}
func (t Hand_typesRepository)  DeleteHand_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Hand_typesRepository -> DeleteHand_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteHand_typesById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_HAND_TYPES_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Hand_typesRepository.repository.DeleteHand_typesById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteHand_typesById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteHand_typesById.deleted", result)
	return true, err
}
func (t Hand_typesRepository)  InsertHand_types(ctx context.Context,hand_types *model.Hand_types) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Hand_typesRepository -> InsertHand_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertHand_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertHand_types.typecode", hand_types.TypeCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_HAND_TYPES_INSERT,
			hand_types.TypeCode,
			hand_types.Name,
			hand_types.Description,
			hand_types.IsActive,
			hand_types.CreatedAt,
	).Scan(&hand_types.ID)

	if err != nil {
		t.log.Error(ctx, "Hand_typesRepository.repository.InsertHand_types.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertHand_types.inserted_id", hand_types.ID)
   return hand_types.ID, nil

}
func (t Hand_typesRepository)  UpdateHand_types(ctx context.Context,hand_types *model.Hand_types, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Hand_typesRepository -> UpdateHand_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateHand_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateHand_types.id", id)
	tracker.AddParam("repository.UpdateHand_types.typecode", hand_types.TypeCode)

	hand_types.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_HAND_TYPES_UPDATE, 
			hand_types.TypeCode,
			hand_types.Name,
			hand_types.Description,
			hand_types.IsActive,
			hand_types.CreatedAt,
			hand_types.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Hand_typesRepository.repository.UpdateHand_types.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateHand_types.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateHand_types.rows_affected", rowsAffected)
	return nil
}

