package gatewaysSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	gatewaysRepo "palm-pay/app/gateways/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type GatewaysServiceIF interface {
     GetGateways(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetGatewaysById(ctx context.Context, id int64) (*model.Gateways, error)
     GetGatewaysByGatewayCode(ctx context.Context, gatewaycode string) (*model.Gateways, error)
     InsertGateways(ctx context.Context, gateways *model.Gateways) (int64, error)
     UpdateGateways(ctx context.Context, gateways *model.Gateways, id int64) error
     DeleteGatewaysById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     gatewaysRepo gatewaysRepo.GatewaysRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewGatewaysService(gatewaysRepo gatewaysRepo.GatewaysRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         gatewaysRepo: gatewaysRepo,
		  observability:  observabilidade.NewServiceObservability("service.gateways"),
     }
}
func (r Resource)  GetGateways(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("GatewaysService -> GetGateways", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetGateways.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetGateways.offset", offset)
	tracker.AddParam("service.GetGateways.limit", limit)



	itemsPage, err = r.gatewaysRepo.GetGateways(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Gateways); ok {
		tracker.AddResult("service.GetGateways.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetGatewaysById(ctx context.Context, id int64) (gateways *model.Gateways, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("GatewaysService -> GetGatewaysById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetGatewaysById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetGatewaysById.id", id)
	gateways, err = r.gatewaysRepo.GetGatewaysById(ctx, id)
	if err != nil {
		return gateways, errors.New(app.MsgRepositoryError)
	}

	return gateways, nil
}
func (r Resource)  GetGatewaysByGatewayCode(ctx context.Context, gatewaycode string) (gateways *model.Gateways, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("GatewaysService -> GetGatewaysByGatewayCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetGatewaysByGatewayCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetGatewaysByGatewayCode.gatewaycode", gatewaycode)
	gateways, err = r.gatewaysRepo.GetGatewaysByGatewayCode(ctx, gatewaycode)
	if err != nil {
		return gateways, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetGatewaysByGatewayCode.found", gateways != nil)
	return gateways, nil
}
func (r Resource)  DeleteGatewaysById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("GatewaysService -> DeleteGatewaysById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteGatewaysById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteGatewaysById.id",id)

	result, err = r.gatewaysRepo.DeleteGatewaysById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteGatewaysById.deleted", result)
	return result, nil
}
func (r Resource)  InsertGateways(ctx context.Context,gateways *model.Gateways) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("GatewaysService -> InsertGateways", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertGateways")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertGateways.gatewaycode", gateways.GatewayCode)
	insertedId, err = r.gatewaysRepo.InsertGateways(ctx, gateways)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertGateways.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateGateways(ctx context.Context,gateways *model.Gateways, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("GatewaysService -> UpdateGateways", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateGateways")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateGateways.id", id)
	tracker.AddParam("service.UpdateGateways.gatewaycode", gateways.GatewayCode)

	err = r.gatewaysRepo.UpdateGateways(ctx, gateways, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateGateways.updated", true)

	return nil
}

