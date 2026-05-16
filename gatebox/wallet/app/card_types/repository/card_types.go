package card_typesRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Card_typesRepositoryIF interface {
     GetCard_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetCard_typesById(ctx context.Context, id int64) (*model.Card_types, error)
     GetCard_typesByTypeCode(ctx context.Context, typecode string) (*model.Card_types, error)
     InsertCard_types(ctx context.Context, card_types *model.Card_types) (int64, error)
     UpdateCard_types(ctx context.Context, card_types *model.Card_types, id int64) error
     DeleteCard_typesById(ctx context.Context, id int64) (bool, error)
}
 type Card_typesRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewCard_typesRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Card_typesRepository{
    return &Card_typesRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Card_types"),
     }
}
func (t Card_typesRepository)  GetCard_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Card_typesRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetCard_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetCard_types.offset", offset)
	tracker.AddParam("repository.GetCard_types.limit", limit)
	itemsPage 			= model.ItemsPage{}
	card_typess := []model.Card_types{}

	rows, err := t.PGRead.Query(ctx, SQL_CARD_TYPES_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Card_typesRepository.repository.GetCard_typess.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var card_types model.Card_types
		err := rows.Scan(
			&card_types.ID,
			&card_types.TypeCode,
			&card_types.Name,
			&card_types.Description,
			&card_types.DefaultDailyLimit,
			&card_types.DefaultMonthlyLimit,
			&card_types.IsActive,
			&card_types.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Card_typesRepository.repository.GetCard_typess.Scan: ", err.Error())
			return itemsPage, err
		}
		card_typess = append(card_typess, card_types)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Card_typesRepository.repository.GetCard_typess.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(card_typess) > 0 {
		qtyRecords = card_typess[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = card_typess

	tracker.AddResult("repository.GetCard_types.rows_returned", len(card_typess))
	tracker.AddResult("repository.GetCard_types.total_count", len(card_typess))

	return itemsPage, nil
}
func (t Card_typesRepository)  GetCard_typesById(ctx context.Context, id int64) (card_types *model.Card_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Card_typesRepository -> GetCard_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetCard_typesById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetCard_typesById.id", id)

	card_types = new(model.Card_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_CARD_TYPES_BY_ID, id)
		err = row.Scan(
			&card_types.ID,
			&card_types.TypeCode,
			&card_types.Name,
			&card_types.Description,
			&card_types.DefaultDailyLimit,
			&card_types.DefaultMonthlyLimit,
			&card_types.IsActive,
			&card_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Card_typesRepository.repository.GetCard_typesById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetCard_typesById.found", true)
	return card_types, nil
}
func (t Card_typesRepository)  GetCard_typesByTypeCode(ctx context.Context, typecode string) (card_types *model.Card_types, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Card_typesRepository -> GetCard_typesByTypeCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetCard_typesByTypeCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetCard_typesByTypeCode.typecode", typecode)

	card_types = new(model.Card_types)
	row := t.PGRead.QueryRow(ctx, SQL_GET_CARD_TYPES_BY_TYPE_CODE, typecode)
		err = row.Scan(
			&card_types.ID,
			&card_types.TypeCode,
			&card_types.Name,
			&card_types.Description,
			&card_types.DefaultDailyLimit,
			&card_types.DefaultMonthlyLimit,
			&card_types.IsActive,
			&card_types.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Card_typesRepository.repository.GetCard_typesBytypecode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return card_types, nil
}
func (t Card_typesRepository)  DeleteCard_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Card_typesRepository -> DeleteCard_typesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteCard_typesById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_CARD_TYPES_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Card_typesRepository.repository.DeleteCard_typesById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteCard_typesById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteCard_typesById.deleted", result)
	return true, err
}
func (t Card_typesRepository)  InsertCard_types(ctx context.Context,card_types *model.Card_types) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Card_typesRepository -> InsertCard_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertCard_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertCard_types.typecode", card_types.TypeCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_CARD_TYPES_INSERT,
			card_types.TypeCode,
			card_types.Name,
			card_types.Description,
			card_types.DefaultDailyLimit,
			card_types.DefaultMonthlyLimit,
			card_types.IsActive,
			card_types.CreatedAt,
	).Scan(&card_types.ID)

	if err != nil {
		t.log.Error(ctx, "Card_typesRepository.repository.InsertCard_types.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertCard_types.inserted_id", card_types.ID)
   return card_types.ID, nil

}
func (t Card_typesRepository)  UpdateCard_types(ctx context.Context,card_types *model.Card_types, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Card_typesRepository -> UpdateCard_types", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateCard_types")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateCard_types.id", id)
	tracker.AddParam("repository.UpdateCard_types.typecode", card_types.TypeCode)

	card_types.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_CARD_TYPES_UPDATE, 
			card_types.TypeCode,
			card_types.Name,
			card_types.Description,
			card_types.DefaultDailyLimit,
			card_types.DefaultMonthlyLimit,
			card_types.IsActive,
			card_types.CreatedAt,
			card_types.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Card_typesRepository.repository.UpdateCard_types.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateCard_types.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateCard_types.rows_affected", rowsAffected)
	return nil
}

