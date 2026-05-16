package gateway_status_typesSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	gateway_status_typesRepo "palm-pay/app/gateway_status_types/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Gateway_status_typesServiceIF interface {
     GetGateway_status_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetGateway_status_typesById(ctx context.Context, id int64) (*model.Gateway_status_types, error)
     GetGateway_status_typesByStatusCode(ctx context.Context, statuscode string) (*model.Gateway_status_types, error)
     InsertGateway_status_types(ctx context.Context, gateway_status_types *model.Gateway_status_types) (int64, error)
     UpdateGateway_status_types(ctx context.Context, gateway_status_types *model.Gateway_status_types, id int64) error
     DeleteGateway_status_typesById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     gateway_status_typesRepo gateway_status_typesRepo.Gateway_status_typesRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewGateway_status_typesService(gateway_status_typesRepo gateway_status_typesRepo.Gateway_status_typesRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         gateway_status_typesRepo: gateway_status_typesRepo,
		  observability:  observabilidade.NewServiceObservability("service.gateway_status_types"),
     }
}
func (r Resource)  GetGateway_status_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Gateway_status_typesService -> GetGateway_status_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetGateway_status_types.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetGateway_status_types.offset", offset)
	tracker.AddParam("service.GetGateway_status_types.limit", limit)



	itemsPage, err = r.gateway_status_typesRepo.GetGateway_status_types(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Gateway_status_types); ok {
		tracker.AddResult("service.GetGateway_status_types.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetGateway_status_typesById(ctx context.Context, id int64) (gateway_status_types *model.Gateway_status_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Gateway_status_typesService -> GetGateway_status_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetGateway_status_typesById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetGateway_status_typesById.id", id)
	gateway_status_types, err = r.gateway_status_typesRepo.GetGateway_status_typesById(ctx, id)
	if err != nil {
		return gateway_status_types, errors.New(app.MsgRepositoryError)
	}

	return gateway_status_types, nil
}
func (r Resource)  GetGateway_status_typesByStatusCode(ctx context.Context, statuscode string) (gateway_status_types *model.Gateway_status_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Gateway_status_typesService -> GetGateway_status_typesByStatusCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetGateway_status_typesByStatusCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetGateway_status_typesByStatusCode.statuscode", statuscode)
	gateway_status_types, err = r.gateway_status_typesRepo.GetGateway_status_typesByStatusCode(ctx, statuscode)
	if err != nil {
		return gateway_status_types, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetGateway_status_typesByStatusCode.found", gateway_status_types != nil)
	return gateway_status_types, nil
}
func (r Resource)  DeleteGateway_status_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Gateway_status_typesService -> DeleteGateway_status_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteGateway_status_typesById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteGateway_status_typesById.id",id)

	result, err = r.gateway_status_typesRepo.DeleteGateway_status_typesById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteGateway_status_typesById.deleted", result)
	return result, nil
}
func (r Resource)  InsertGateway_status_types(ctx context.Context,gateway_status_types *model.Gateway_status_types) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Gateway_status_typesService -> InsertGateway_status_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertGateway_status_types")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertGateway_status_types.statuscode", gateway_status_types.StatusCode)
	insertedId, err = r.gateway_status_typesRepo.InsertGateway_status_types(ctx, gateway_status_types)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertGateway_status_types.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateGateway_status_types(ctx context.Context,gateway_status_types *model.Gateway_status_types, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Gateway_status_typesService -> UpdateGateway_status_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateGateway_status_types")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateGateway_status_types.id", id)
	tracker.AddParam("service.UpdateGateway_status_types.statuscode", gateway_status_types.StatusCode)

	err = r.gateway_status_typesRepo.UpdateGateway_status_types(ctx, gateway_status_types, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateGateway_status_types.updated", true)

	return nil
}

