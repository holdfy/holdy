package payment_methodsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	payment_methodsRepo "palm-pay/app/payment_methods/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Payment_methodsServiceIF interface {
     GetPayment_methods(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetPayment_methodsById(ctx context.Context, id int64) (*model.Payment_methods, error)
     GetPayment_methodsByMethodCode(ctx context.Context, methodcode string) (*model.Payment_methods, error)
     InsertPayment_methods(ctx context.Context, payment_methods *model.Payment_methods) (int64, error)
     UpdatePayment_methods(ctx context.Context, payment_methods *model.Payment_methods, id int64) error
     DeletePayment_methodsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     payment_methodsRepo payment_methodsRepo.Payment_methodsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewPayment_methodsService(payment_methodsRepo payment_methodsRepo.Payment_methodsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         payment_methodsRepo: payment_methodsRepo,
		  observability:  observabilidade.NewServiceObservability("service.payment_methods"),
     }
}
func (r Resource)  GetPayment_methods(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Payment_methodsService -> GetPayment_methods", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetPayment_methods.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetPayment_methods.offset", offset)
	tracker.AddParam("service.GetPayment_methods.limit", limit)



	itemsPage, err = r.payment_methodsRepo.GetPayment_methods(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Payment_methods); ok {
		tracker.AddResult("service.GetPayment_methods.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetPayment_methodsById(ctx context.Context, id int64) (payment_methods *model.Payment_methods, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Payment_methodsService -> GetPayment_methodsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetPayment_methodsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetPayment_methodsById.id", id)
	payment_methods, err = r.payment_methodsRepo.GetPayment_methodsById(ctx, id)
	if err != nil {
		return payment_methods, errors.New(app.MsgRepositoryError)
	}

	return payment_methods, nil
}
func (r Resource)  GetPayment_methodsByMethodCode(ctx context.Context, methodcode string) (payment_methods *model.Payment_methods, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Payment_methodsService -> GetPayment_methodsByMethodCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetPayment_methodsByMethodCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetPayment_methodsByMethodCode.methodcode", methodcode)
	payment_methods, err = r.payment_methodsRepo.GetPayment_methodsByMethodCode(ctx, methodcode)
	if err != nil {
		return payment_methods, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetPayment_methodsByMethodCode.found", payment_methods != nil)
	return payment_methods, nil
}
func (r Resource)  DeletePayment_methodsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Payment_methodsService -> DeletePayment_methodsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeletePayment_methodsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeletePayment_methodsById.id",id)

	result, err = r.payment_methodsRepo.DeletePayment_methodsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeletePayment_methodsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertPayment_methods(ctx context.Context,payment_methods *model.Payment_methods) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Payment_methodsService -> InsertPayment_methods", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertPayment_methods")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertPayment_methods.methodcode", payment_methods.MethodCode)
	insertedId, err = r.payment_methodsRepo.InsertPayment_methods(ctx, payment_methods)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertPayment_methods.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdatePayment_methods(ctx context.Context,payment_methods *model.Payment_methods, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Payment_methodsService -> UpdatePayment_methods", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdatePayment_methods")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdatePayment_methods.id", id)
	tracker.AddParam("service.UpdatePayment_methods.methodcode", payment_methods.MethodCode)

	err = r.payment_methodsRepo.UpdatePayment_methods(ctx, payment_methods, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdatePayment_methods.updated", true)

	return nil
}

