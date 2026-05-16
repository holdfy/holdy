package transaction_status_historySV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	transaction_status_historyRepo "palm-pay/app/transaction_status_history/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Transaction_status_historyServiceIF interface {
     GetTransaction_status_history(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetTransaction_status_historyById(ctx context.Context, id int64) (*model.Transaction_status_history, error)
     GetTransaction_status_historyByStatusHistoryCode(ctx context.Context, statushistorycode string) (*model.Transaction_status_history, error)
     InsertTransaction_status_history(ctx context.Context, transaction_status_history *model.Transaction_status_history) (int64, error)
     UpdateTransaction_status_history(ctx context.Context, transaction_status_history *model.Transaction_status_history, id int64) error
     DeleteTransaction_status_historyById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     transaction_status_historyRepo transaction_status_historyRepo.Transaction_status_historyRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewTransaction_status_historyService(transaction_status_historyRepo transaction_status_historyRepo.Transaction_status_historyRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         transaction_status_historyRepo: transaction_status_historyRepo,
		  observability:  observabilidade.NewServiceObservability("service.transaction_status_history"),
     }
}
func (r Resource)  GetTransaction_status_history(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_status_historyService -> GetTransaction_status_history", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetTransaction_status_history.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetTransaction_status_history.offset", offset)
	tracker.AddParam("service.GetTransaction_status_history.limit", limit)



	itemsPage, err = r.transaction_status_historyRepo.GetTransaction_status_history(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Transaction_status_history); ok {
		tracker.AddResult("service.GetTransaction_status_history.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetTransaction_status_historyById(ctx context.Context, id int64) (transaction_status_history *model.Transaction_status_history, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_status_historyService -> GetTransaction_status_historyById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetTransaction_status_historyById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetTransaction_status_historyById.id", id)
	transaction_status_history, err = r.transaction_status_historyRepo.GetTransaction_status_historyById(ctx, id)
	if err != nil {
		return transaction_status_history, errors.New(app.MsgRepositoryError)
	}

	return transaction_status_history, nil
}
func (r Resource)  GetTransaction_status_historyByStatusHistoryCode(ctx context.Context, statushistorycode string) (transaction_status_history *model.Transaction_status_history, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_status_historyService -> GetTransaction_status_historyByStatusHistoryCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetTransaction_status_historyByStatusHistoryCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetTransaction_status_historyByStatusHistoryCode.statushistorycode", statushistorycode)
	transaction_status_history, err = r.transaction_status_historyRepo.GetTransaction_status_historyByStatusHistoryCode(ctx, statushistorycode)
	if err != nil {
		return transaction_status_history, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetTransaction_status_historyByStatusHistoryCode.found", transaction_status_history != nil)
	return transaction_status_history, nil
}
func (r Resource)  DeleteTransaction_status_historyById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_status_historyService -> DeleteTransaction_status_historyById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteTransaction_status_historyById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteTransaction_status_historyById.id",id)

	result, err = r.transaction_status_historyRepo.DeleteTransaction_status_historyById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteTransaction_status_historyById.deleted", result)
	return result, nil
}
func (r Resource)  InsertTransaction_status_history(ctx context.Context,transaction_status_history *model.Transaction_status_history) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_status_historyService -> InsertTransaction_status_history", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertTransaction_status_history")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertTransaction_status_history.statushistorycode", transaction_status_history.StatusHistoryCode)
	insertedId, err = r.transaction_status_historyRepo.InsertTransaction_status_history(ctx, transaction_status_history)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertTransaction_status_history.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateTransaction_status_history(ctx context.Context,transaction_status_history *model.Transaction_status_history, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_status_historyService -> UpdateTransaction_status_history", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateTransaction_status_history")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateTransaction_status_history.id", id)
	tracker.AddParam("service.UpdateTransaction_status_history.statushistorycode", transaction_status_history.StatusHistoryCode)

	err = r.transaction_status_historyRepo.UpdateTransaction_status_history(ctx, transaction_status_history, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateTransaction_status_history.updated", true)

	return nil
}

