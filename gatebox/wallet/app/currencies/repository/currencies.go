package currenciesRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type CurrenciesRepositoryIF interface {
     GetCurrencies(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetCurrenciesById(ctx context.Context, id int64) (*model.Currencies, error)
     GetCurrenciesByCurrencyCode(ctx context.Context, currencycode string) (*model.Currencies, error)
     InsertCurrencies(ctx context.Context, currencies *model.Currencies) (int64, error)
     UpdateCurrencies(ctx context.Context, currencies *model.Currencies, id int64) error
     DeleteCurrenciesById(ctx context.Context, id int64) (bool, error)
}
 type CurrenciesRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewCurrenciesRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *CurrenciesRepository{
    return &CurrenciesRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Currencies"),
     }
}
func (t CurrenciesRepository)  GetCurrencies(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("CurrenciesRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetCurrencies")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetCurrencies.offset", offset)
	tracker.AddParam("repository.GetCurrencies.limit", limit)
	itemsPage 			= model.ItemsPage{}
	currenciess := []model.Currencies{}

	rows, err := t.PGRead.Query(ctx, SQL_CURRENCIES_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "CurrenciesRepository.repository.GetCurrenciess.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var currencies model.Currencies
		err := rows.Scan(
			&currencies.ID,
			&currencies.CurrencyCode,
			&currencies.IsoCode,
			&currencies.Name,
			&currencies.Symbol,
			&currencies.DecimalPlaces,
			&currencies.IsActive,
			&currencies.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "CurrenciesRepository.repository.GetCurrenciess.Scan: ", err.Error())
			return itemsPage, err
		}
		currenciess = append(currenciess, currencies)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "CurrenciesRepository.repository.GetCurrenciess.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(currenciess) > 0 {
		qtyRecords = currenciess[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = currenciess

	tracker.AddResult("repository.GetCurrencies.rows_returned", len(currenciess))
	tracker.AddResult("repository.GetCurrencies.total_count", len(currenciess))

	return itemsPage, nil
}
func (t CurrenciesRepository)  GetCurrenciesById(ctx context.Context, id int64) (currencies *model.Currencies, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("CurrenciesRepository -> GetCurrenciesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetCurrenciesById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetCurrenciesById.id", id)

	currencies = new(model.Currencies)
	row := t.PGRead.QueryRow(ctx, SQL_GET_CURRENCIES_BY_ID, id)
		err = row.Scan(
			&currencies.ID,
			&currencies.CurrencyCode,
			&currencies.IsoCode,
			&currencies.Name,
			&currencies.Symbol,
			&currencies.DecimalPlaces,
			&currencies.IsActive,
			&currencies.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"CurrenciesRepository.repository.GetCurrenciesById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetCurrenciesById.found", true)
	return currencies, nil
}
func (t CurrenciesRepository)  GetCurrenciesByCurrencyCode(ctx context.Context, currencycode string) (currencies *model.Currencies, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("CurrenciesRepository -> GetCurrenciesByCurrencyCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetCurrenciesByCurrencyCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetCurrenciesByCurrencyCode.currencycode", currencycode)

	currencies = new(model.Currencies)
	row := t.PGRead.QueryRow(ctx, SQL_GET_CURRENCIES_BY_CURRENCY_CODE, currencycode)
		err = row.Scan(
			&currencies.ID,
			&currencies.CurrencyCode,
			&currencies.IsoCode,
			&currencies.Name,
			&currencies.Symbol,
			&currencies.DecimalPlaces,
			&currencies.IsActive,
			&currencies.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"CurrenciesRepository.repository.GetCurrenciesBycurrencycode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return currencies, nil
}
func (t CurrenciesRepository)  DeleteCurrenciesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("CurrenciesRepository -> DeleteCurrenciesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteCurrenciesById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_CURRENCIES_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"CurrenciesRepository.repository.DeleteCurrenciesById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteCurrenciesById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteCurrenciesById.deleted", result)
	return true, err
}
func (t CurrenciesRepository)  InsertCurrencies(ctx context.Context,currencies *model.Currencies) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("CurrenciesRepository -> InsertCurrencies", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertCurrencies")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertCurrencies.currencycode", currencies.CurrencyCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_CURRENCIES_INSERT,
			currencies.CurrencyCode,
			currencies.IsoCode,
			currencies.Name,
			currencies.Symbol,
			currencies.DecimalPlaces,
			currencies.IsActive,
			currencies.CreatedAt,
	).Scan(&currencies.ID)

	if err != nil {
		t.log.Error(ctx, "CurrenciesRepository.repository.InsertCurrencies.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertCurrencies.inserted_id", currencies.ID)
   return currencies.ID, nil

}
func (t CurrenciesRepository)  UpdateCurrencies(ctx context.Context,currencies *model.Currencies, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("CurrenciesRepository -> UpdateCurrencies", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateCurrencies")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateCurrencies.id", id)
	tracker.AddParam("repository.UpdateCurrencies.currencycode", currencies.CurrencyCode)

	currencies.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_CURRENCIES_UPDATE, 
			currencies.CurrencyCode,
			currencies.IsoCode,
			currencies.Name,
			currencies.Symbol,
			currencies.DecimalPlaces,
			currencies.IsActive,
			currencies.CreatedAt,
			currencies.ID,
   )
	if err != nil {
		t.log.Error(ctx, "CurrenciesRepository.repository.UpdateCurrencies.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateCurrencies.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateCurrencies.rows_affected", rowsAffected)
	return nil
}

