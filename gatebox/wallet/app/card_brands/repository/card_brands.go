package card_brandsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Card_brandsRepositoryIF interface {
     GetCard_brands(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetCard_brandsById(ctx context.Context, id int64) (*model.Card_brands, error)
     GetCard_brandsByBrandCode(ctx context.Context, brandcode string) (*model.Card_brands, error)
     InsertCard_brands(ctx context.Context, card_brands *model.Card_brands) (int64, error)
     UpdateCard_brands(ctx context.Context, card_brands *model.Card_brands, id int64) error
     DeleteCard_brandsById(ctx context.Context, id int64) (bool, error)
}
 type Card_brandsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewCard_brandsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Card_brandsRepository{
    return &Card_brandsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Card_brands"),
     }
}
func (t Card_brandsRepository)  GetCard_brands(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Card_brandsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetCard_brands")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetCard_brands.offset", offset)
	tracker.AddParam("repository.GetCard_brands.limit", limit)
	itemsPage 			= model.ItemsPage{}
	card_brandss := []model.Card_brands{}

	rows, err := t.PGRead.Query(ctx, SQL_CARD_BRANDS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Card_brandsRepository.repository.GetCard_brandss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var card_brands model.Card_brands
		err := rows.Scan(
			&card_brands.ID,
			&card_brands.BrandCode,
			&card_brands.Name,
			&card_brands.Description,
			&card_brands.LogoUrl,
			&card_brands.IsActive,
			&card_brands.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Card_brandsRepository.repository.GetCard_brandss.Scan: ", err.Error())
			return itemsPage, err
		}
		card_brandss = append(card_brandss, card_brands)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Card_brandsRepository.repository.GetCard_brandss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(card_brandss) > 0 {
		qtyRecords = card_brandss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = card_brandss

	tracker.AddResult("repository.GetCard_brands.rows_returned", len(card_brandss))
	tracker.AddResult("repository.GetCard_brands.total_count", len(card_brandss))

	return itemsPage, nil
}
func (t Card_brandsRepository)  GetCard_brandsById(ctx context.Context, id int64) (card_brands *model.Card_brands, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Card_brandsRepository -> GetCard_brandsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetCard_brandsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetCard_brandsById.id", id)

	card_brands = new(model.Card_brands)
	row := t.PGRead.QueryRow(ctx, SQL_GET_CARD_BRANDS_BY_ID, id)
		err = row.Scan(
			&card_brands.ID,
			&card_brands.BrandCode,
			&card_brands.Name,
			&card_brands.Description,
			&card_brands.LogoUrl,
			&card_brands.IsActive,
			&card_brands.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Card_brandsRepository.repository.GetCard_brandsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetCard_brandsById.found", true)
	return card_brands, nil
}
func (t Card_brandsRepository)  GetCard_brandsByBrandCode(ctx context.Context, brandcode string) (card_brands *model.Card_brands, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Card_brandsRepository -> GetCard_brandsByBrandCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetCard_brandsByBrandCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetCard_brandsByBrandCode.brandcode", brandcode)

	card_brands = new(model.Card_brands)
	row := t.PGRead.QueryRow(ctx, SQL_GET_CARD_BRANDS_BY_BRAND_CODE, brandcode)
		err = row.Scan(
			&card_brands.ID,
			&card_brands.BrandCode,
			&card_brands.Name,
			&card_brands.Description,
			&card_brands.LogoUrl,
			&card_brands.IsActive,
			&card_brands.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Card_brandsRepository.repository.GetCard_brandsBybrandcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return card_brands, nil
}
func (t Card_brandsRepository)  DeleteCard_brandsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Card_brandsRepository -> DeleteCard_brandsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteCard_brandsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_CARD_BRANDS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Card_brandsRepository.repository.DeleteCard_brandsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteCard_brandsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteCard_brandsById.deleted", result)
	return true, err
}
func (t Card_brandsRepository)  InsertCard_brands(ctx context.Context,card_brands *model.Card_brands) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Card_brandsRepository -> InsertCard_brands", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertCard_brands")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertCard_brands.brandcode", card_brands.BrandCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_CARD_BRANDS_INSERT,
			card_brands.BrandCode,
			card_brands.Name,
			card_brands.Description,
			card_brands.LogoUrl,
			card_brands.IsActive,
			card_brands.CreatedAt,
	).Scan(&card_brands.ID)

	if err != nil {
		t.log.Error(ctx, "Card_brandsRepository.repository.InsertCard_brands.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertCard_brands.inserted_id", card_brands.ID)
   return card_brands.ID, nil

}
func (t Card_brandsRepository)  UpdateCard_brands(ctx context.Context,card_brands *model.Card_brands, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Card_brandsRepository -> UpdateCard_brands", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateCard_brands")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateCard_brands.id", id)
	tracker.AddParam("repository.UpdateCard_brands.brandcode", card_brands.BrandCode)

	card_brands.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_CARD_BRANDS_UPDATE, 
			card_brands.BrandCode,
			card_brands.Name,
			card_brands.Description,
			card_brands.LogoUrl,
			card_brands.IsActive,
			card_brands.CreatedAt,
			card_brands.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Card_brandsRepository.repository.UpdateCard_brands.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateCard_brands.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateCard_brands.rows_affected", rowsAffected)
	return nil
}

