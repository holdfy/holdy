package document_typesSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	document_typesRepo "palm-pay/app/document_types/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Document_typesServiceIF interface {
     GetDocument_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetDocument_typesById(ctx context.Context, id int64) (*model.Document_types, error)
     GetDocument_typesByTypeCode(ctx context.Context, typecode string) (*model.Document_types, error)
     InsertDocument_types(ctx context.Context, document_types *model.Document_types) (int64, error)
     UpdateDocument_types(ctx context.Context, document_types *model.Document_types, id int64) error
     DeleteDocument_typesById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     document_typesRepo document_typesRepo.Document_typesRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewDocument_typesService(document_typesRepo document_typesRepo.Document_typesRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         document_typesRepo: document_typesRepo,
		  observability:  observabilidade.NewServiceObservability("service.document_types"),
     }
}
func (r Resource)  GetDocument_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Document_typesService -> GetDocument_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetDocument_types.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetDocument_types.offset", offset)
	tracker.AddParam("service.GetDocument_types.limit", limit)



	itemsPage, err = r.document_typesRepo.GetDocument_types(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Document_types); ok {
		tracker.AddResult("service.GetDocument_types.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetDocument_typesById(ctx context.Context, id int64) (document_types *model.Document_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Document_typesService -> GetDocument_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetDocument_typesById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetDocument_typesById.id", id)
	document_types, err = r.document_typesRepo.GetDocument_typesById(ctx, id)
	if err != nil {
		return document_types, errors.New(app.MsgRepositoryError)
	}

	return document_types, nil
}
func (r Resource)  GetDocument_typesByTypeCode(ctx context.Context, typecode string) (document_types *model.Document_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Document_typesService -> GetDocument_typesByTypeCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetDocument_typesByTypeCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetDocument_typesByTypeCode.typecode", typecode)
	document_types, err = r.document_typesRepo.GetDocument_typesByTypeCode(ctx, typecode)
	if err != nil {
		return document_types, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetDocument_typesByTypeCode.found", document_types != nil)
	return document_types, nil
}
func (r Resource)  DeleteDocument_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Document_typesService -> DeleteDocument_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteDocument_typesById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteDocument_typesById.id",id)

	result, err = r.document_typesRepo.DeleteDocument_typesById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteDocument_typesById.deleted", result)
	return result, nil
}
func (r Resource)  InsertDocument_types(ctx context.Context,document_types *model.Document_types) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Document_typesService -> InsertDocument_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertDocument_types")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertDocument_types.typecode", document_types.TypeCode)
	insertedId, err = r.document_typesRepo.InsertDocument_types(ctx, document_types)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertDocument_types.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateDocument_types(ctx context.Context,document_types *model.Document_types, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Document_typesService -> UpdateDocument_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateDocument_types")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateDocument_types.id", id)
	tracker.AddParam("service.UpdateDocument_types.typecode", document_types.TypeCode)

	err = r.document_typesRepo.UpdateDocument_types(ctx, document_types, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateDocument_types.updated", true)

	return nil
}

