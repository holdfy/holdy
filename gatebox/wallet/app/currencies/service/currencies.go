package currenciesSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	currenciesRepo "palm-pay/app/currencies/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type CurrenciesServiceIF interface {
     GetCurrencies(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetCurrenciesById(ctx context.Context, id int64) (*model.Currencies, error)
     GetCurrenciesByCurrencyCode(ctx context.Context, currencycode string) (*model.Currencies, error)
     InsertCurrencies(ctx context.Context, currencies *model.Currencies) (int64, error)
     UpdateCurrencies(ctx context.Context, currencies *model.Currencies, id int64) error
     DeleteCurrenciesById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     currenciesRepo currenciesRepo.CurrenciesRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewCurrenciesService(currenciesRepo currenciesRepo.CurrenciesRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         currenciesRepo: currenciesRepo,
		  observability:  observabilidade.NewServiceObservability("service.currencies"),
     }
}
func (r Resource)  GetCurrencies(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("CurrenciesService -> GetCurrencies", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetCurrencies.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetCurrencies.offset", offset)
	tracker.AddParam("service.GetCurrencies.limit", limit)



	itemsPage, err = r.currenciesRepo.GetCurrencies(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Currencies); ok {
		tracker.AddResult("service.GetCurrencies.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetCurrenciesById(ctx context.Context, id int64) (currencies *model.Currencies, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("CurrenciesService -> GetCurrenciesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetCurrenciesById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetCurrenciesById.id", id)
	currencies, err = r.currenciesRepo.GetCurrenciesById(ctx, id)
	if err != nil {
		return currencies, errors.New(app.MsgRepositoryError)
	}

	return currencies, nil
}
func (r Resource)  GetCurrenciesByCurrencyCode(ctx context.Context, currencycode string) (currencies *model.Currencies, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("CurrenciesService -> GetCurrenciesByCurrencyCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetCurrenciesByCurrencyCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetCurrenciesByCurrencyCode.currencycode", currencycode)
	currencies, err = r.currenciesRepo.GetCurrenciesByCurrencyCode(ctx, currencycode)
	if err != nil {
		return currencies, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetCurrenciesByCurrencyCode.found", currencies != nil)
	return currencies, nil
}
func (r Resource)  DeleteCurrenciesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("CurrenciesService -> DeleteCurrenciesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteCurrenciesById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteCurrenciesById.id",id)

	result, err = r.currenciesRepo.DeleteCurrenciesById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteCurrenciesById.deleted", result)
	return result, nil
}
func (r Resource)  InsertCurrencies(ctx context.Context,currencies *model.Currencies) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("CurrenciesService -> InsertCurrencies", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertCurrencies")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertCurrencies.currencycode", currencies.CurrencyCode)
	insertedId, err = r.currenciesRepo.InsertCurrencies(ctx, currencies)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertCurrencies.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateCurrencies(ctx context.Context,currencies *model.Currencies, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("CurrenciesService -> UpdateCurrencies", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateCurrencies")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateCurrencies.id", id)
	tracker.AddParam("service.UpdateCurrencies.currencycode", currencies.CurrencyCode)

	err = r.currenciesRepo.UpdateCurrencies(ctx, currencies, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateCurrencies.updated", true)

	return nil
}

