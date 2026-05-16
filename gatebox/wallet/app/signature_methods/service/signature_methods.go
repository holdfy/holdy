package signature_methodsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	signature_methodsRepo "palm-pay/app/signature_methods/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Signature_methodsServiceIF interface {
     GetSignature_methods(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetSignature_methodsById(ctx context.Context, id int64) (*model.Signature_methods, error)
     GetSignature_methodsByMethodCode(ctx context.Context, methodcode string) (*model.Signature_methods, error)
     InsertSignature_methods(ctx context.Context, signature_methods *model.Signature_methods) (int64, error)
     UpdateSignature_methods(ctx context.Context, signature_methods *model.Signature_methods, id int64) error
     DeleteSignature_methodsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     signature_methodsRepo signature_methodsRepo.Signature_methodsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewSignature_methodsService(signature_methodsRepo signature_methodsRepo.Signature_methodsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         signature_methodsRepo: signature_methodsRepo,
		  observability:  observabilidade.NewServiceObservability("service.signature_methods"),
     }
}
func (r Resource)  GetSignature_methods(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Signature_methodsService -> GetSignature_methods", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetSignature_methods.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetSignature_methods.offset", offset)
	tracker.AddParam("service.GetSignature_methods.limit", limit)



	itemsPage, err = r.signature_methodsRepo.GetSignature_methods(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Signature_methods); ok {
		tracker.AddResult("service.GetSignature_methods.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetSignature_methodsById(ctx context.Context, id int64) (signature_methods *model.Signature_methods, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Signature_methodsService -> GetSignature_methodsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetSignature_methodsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetSignature_methodsById.id", id)
	signature_methods, err = r.signature_methodsRepo.GetSignature_methodsById(ctx, id)
	if err != nil {
		return signature_methods, errors.New(app.MsgRepositoryError)
	}

	return signature_methods, nil
}
func (r Resource)  GetSignature_methodsByMethodCode(ctx context.Context, methodcode string) (signature_methods *model.Signature_methods, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Signature_methodsService -> GetSignature_methodsByMethodCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetSignature_methodsByMethodCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetSignature_methodsByMethodCode.methodcode", methodcode)
	signature_methods, err = r.signature_methodsRepo.GetSignature_methodsByMethodCode(ctx, methodcode)
	if err != nil {
		return signature_methods, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetSignature_methodsByMethodCode.found", signature_methods != nil)
	return signature_methods, nil
}
func (r Resource)  DeleteSignature_methodsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Signature_methodsService -> DeleteSignature_methodsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteSignature_methodsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteSignature_methodsById.id",id)

	result, err = r.signature_methodsRepo.DeleteSignature_methodsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteSignature_methodsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertSignature_methods(ctx context.Context,signature_methods *model.Signature_methods) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Signature_methodsService -> InsertSignature_methods", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertSignature_methods")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertSignature_methods.methodcode", signature_methods.MethodCode)
	insertedId, err = r.signature_methodsRepo.InsertSignature_methods(ctx, signature_methods)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertSignature_methods.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateSignature_methods(ctx context.Context,signature_methods *model.Signature_methods, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Signature_methodsService -> UpdateSignature_methods", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateSignature_methods")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateSignature_methods.id", id)
	tracker.AddParam("service.UpdateSignature_methods.methodcode", signature_methods.MethodCode)

	err = r.signature_methodsRepo.UpdateSignature_methods(ctx, signature_methods, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateSignature_methods.updated", true)

	return nil
}

