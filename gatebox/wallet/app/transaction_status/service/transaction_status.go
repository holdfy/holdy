package transaction_statusSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	transaction_statusRepo "palm-pay/app/transaction_status/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Transaction_statusServiceIF interface {
     GetTransaction_status(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetTransaction_statusById(ctx context.Context, id int64) (*model.Transaction_status, error)
     GetTransaction_statusByStatusCode(ctx context.Context, statuscode string) (*model.Transaction_status, error)
     InsertTransaction_status(ctx context.Context, transaction_status *model.Transaction_status) (int64, error)
     UpdateTransaction_status(ctx context.Context, transaction_status *model.Transaction_status, id int64) error
     DeleteTransaction_statusById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     transaction_statusRepo transaction_statusRepo.Transaction_statusRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewTransaction_statusService(transaction_statusRepo transaction_statusRepo.Transaction_statusRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         transaction_statusRepo: transaction_statusRepo,
		  observability:  observabilidade.NewServiceObservability("service.transaction_status"),
     }
}
func (r Resource)  GetTransaction_status(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_statusService -> GetTransaction_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetTransaction_status.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetTransaction_status.offset", offset)
	tracker.AddParam("service.GetTransaction_status.limit", limit)



	itemsPage, err = r.transaction_statusRepo.GetTransaction_status(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Transaction_status); ok {
		tracker.AddResult("service.GetTransaction_status.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetTransaction_statusById(ctx context.Context, id int64) (transaction_status *model.Transaction_status, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_statusService -> GetTransaction_statusById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetTransaction_statusById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetTransaction_statusById.id", id)
	transaction_status, err = r.transaction_statusRepo.GetTransaction_statusById(ctx, id)
	if err != nil {
		return transaction_status, errors.New(app.MsgRepositoryError)
	}

	return transaction_status, nil
}
func (r Resource)  GetTransaction_statusByStatusCode(ctx context.Context, statuscode string) (transaction_status *model.Transaction_status, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_statusService -> GetTransaction_statusByStatusCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetTransaction_statusByStatusCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetTransaction_statusByStatusCode.statuscode", statuscode)
	transaction_status, err = r.transaction_statusRepo.GetTransaction_statusByStatusCode(ctx, statuscode)
	if err != nil {
		return transaction_status, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetTransaction_statusByStatusCode.found", transaction_status != nil)
	return transaction_status, nil
}
func (r Resource)  DeleteTransaction_statusById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_statusService -> DeleteTransaction_statusById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteTransaction_statusById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteTransaction_statusById.id",id)

	result, err = r.transaction_statusRepo.DeleteTransaction_statusById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteTransaction_statusById.deleted", result)
	return result, nil
}
func (r Resource)  InsertTransaction_status(ctx context.Context,transaction_status *model.Transaction_status) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_statusService -> InsertTransaction_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertTransaction_status")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertTransaction_status.statuscode", transaction_status.StatusCode)
	insertedId, err = r.transaction_statusRepo.InsertTransaction_status(ctx, transaction_status)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertTransaction_status.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateTransaction_status(ctx context.Context,transaction_status *model.Transaction_status, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_statusService -> UpdateTransaction_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateTransaction_status")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateTransaction_status.id", id)
	tracker.AddParam("service.UpdateTransaction_status.statuscode", transaction_status.StatusCode)

	err = r.transaction_statusRepo.UpdateTransaction_status(ctx, transaction_status, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateTransaction_status.updated", true)

	return nil
}

