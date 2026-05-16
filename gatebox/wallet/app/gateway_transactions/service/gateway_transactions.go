package gateway_transactionsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	gateway_transactionsRepo "palm-pay/app/gateway_transactions/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Gateway_transactionsServiceIF interface {
     GetGateway_transactions(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetGateway_transactionsById(ctx context.Context, id int64) (*model.Gateway_transactions, error)
     GetGateway_transactionsByGatewayTransactionCode(ctx context.Context, gatewaytransactioncode string) (*model.Gateway_transactions, error)
     InsertGateway_transactions(ctx context.Context, gateway_transactions *model.Gateway_transactions) (int64, error)
     UpdateGateway_transactions(ctx context.Context, gateway_transactions *model.Gateway_transactions, id int64) error
     DeleteGateway_transactionsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     gateway_transactionsRepo gateway_transactionsRepo.Gateway_transactionsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewGateway_transactionsService(gateway_transactionsRepo gateway_transactionsRepo.Gateway_transactionsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         gateway_transactionsRepo: gateway_transactionsRepo,
		  observability:  observabilidade.NewServiceObservability("service.gateway_transactions"),
     }
}
func (r Resource)  GetGateway_transactions(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Gateway_transactionsService -> GetGateway_transactions", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetGateway_transactions.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetGateway_transactions.offset", offset)
	tracker.AddParam("service.GetGateway_transactions.limit", limit)



	itemsPage, err = r.gateway_transactionsRepo.GetGateway_transactions(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Gateway_transactions); ok {
		tracker.AddResult("service.GetGateway_transactions.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetGateway_transactionsById(ctx context.Context, id int64) (gateway_transactions *model.Gateway_transactions, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Gateway_transactionsService -> GetGateway_transactionsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetGateway_transactionsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetGateway_transactionsById.id", id)
	gateway_transactions, err = r.gateway_transactionsRepo.GetGateway_transactionsById(ctx, id)
	if err != nil {
		return gateway_transactions, errors.New(app.MsgRepositoryError)
	}

	return gateway_transactions, nil
}
func (r Resource)  GetGateway_transactionsByGatewayTransactionCode(ctx context.Context, gatewaytransactioncode string) (gateway_transactions *model.Gateway_transactions, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Gateway_transactionsService -> GetGateway_transactionsByGatewayTransactionCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetGateway_transactionsByGatewayTransactionCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetGateway_transactionsByGatewayTransactionCode.gatewaytransactioncode", gatewaytransactioncode)
	gateway_transactions, err = r.gateway_transactionsRepo.GetGateway_transactionsByGatewayTransactionCode(ctx, gatewaytransactioncode)
	if err != nil {
		return gateway_transactions, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetGateway_transactionsByGatewayTransactionCode.found", gateway_transactions != nil)
	return gateway_transactions, nil
}
func (r Resource)  DeleteGateway_transactionsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Gateway_transactionsService -> DeleteGateway_transactionsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteGateway_transactionsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteGateway_transactionsById.id",id)

	result, err = r.gateway_transactionsRepo.DeleteGateway_transactionsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteGateway_transactionsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertGateway_transactions(ctx context.Context,gateway_transactions *model.Gateway_transactions) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Gateway_transactionsService -> InsertGateway_transactions", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertGateway_transactions")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertGateway_transactions.gatewaytransactioncode", gateway_transactions.GatewayTransactionCode)
	insertedId, err = r.gateway_transactionsRepo.InsertGateway_transactions(ctx, gateway_transactions)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertGateway_transactions.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateGateway_transactions(ctx context.Context,gateway_transactions *model.Gateway_transactions, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Gateway_transactionsService -> UpdateGateway_transactions", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateGateway_transactions")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateGateway_transactions.id", id)
	tracker.AddParam("service.UpdateGateway_transactions.gatewaytransactioncode", gateway_transactions.GatewayTransactionCode)

	err = r.gateway_transactionsRepo.UpdateGateway_transactions(ctx, gateway_transactions, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateGateway_transactions.updated", true)

	return nil
}

