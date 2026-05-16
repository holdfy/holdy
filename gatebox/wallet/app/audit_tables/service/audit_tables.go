package audit_tablesSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	audit_tablesRepo "palm-pay/app/audit_tables/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Audit_tablesServiceIF interface {
     GetAudit_tables(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetAudit_tablesById(ctx context.Context, id int64) (*model.Audit_tables, error)
     GetAudit_tablesByTableCode(ctx context.Context, tablecode string) (*model.Audit_tables, error)
     InsertAudit_tables(ctx context.Context, audit_tables *model.Audit_tables) (int64, error)
     UpdateAudit_tables(ctx context.Context, audit_tables *model.Audit_tables, id int64) error
     DeleteAudit_tablesById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     audit_tablesRepo audit_tablesRepo.Audit_tablesRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewAudit_tablesService(audit_tablesRepo audit_tablesRepo.Audit_tablesRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         audit_tablesRepo: audit_tablesRepo,
		  observability:  observabilidade.NewServiceObservability("service.audit_tables"),
     }
}
func (r Resource)  GetAudit_tables(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_tablesService -> GetAudit_tables", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetAudit_tables.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetAudit_tables.offset", offset)
	tracker.AddParam("service.GetAudit_tables.limit", limit)



	itemsPage, err = r.audit_tablesRepo.GetAudit_tables(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Audit_tables); ok {
		tracker.AddResult("service.GetAudit_tables.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetAudit_tablesById(ctx context.Context, id int64) (audit_tables *model.Audit_tables, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_tablesService -> GetAudit_tablesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetAudit_tablesById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetAudit_tablesById.id", id)
	audit_tables, err = r.audit_tablesRepo.GetAudit_tablesById(ctx, id)
	if err != nil {
		return audit_tables, errors.New(app.MsgRepositoryError)
	}

	return audit_tables, nil
}
func (r Resource)  GetAudit_tablesByTableCode(ctx context.Context, tablecode string) (audit_tables *model.Audit_tables, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_tablesService -> GetAudit_tablesByTableCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetAudit_tablesByTableCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetAudit_tablesByTableCode.tablecode", tablecode)
	audit_tables, err = r.audit_tablesRepo.GetAudit_tablesByTableCode(ctx, tablecode)
	if err != nil {
		return audit_tables, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetAudit_tablesByTableCode.found", audit_tables != nil)
	return audit_tables, nil
}
func (r Resource)  DeleteAudit_tablesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_tablesService -> DeleteAudit_tablesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteAudit_tablesById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteAudit_tablesById.id",id)

	result, err = r.audit_tablesRepo.DeleteAudit_tablesById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteAudit_tablesById.deleted", result)
	return result, nil
}
func (r Resource)  InsertAudit_tables(ctx context.Context,audit_tables *model.Audit_tables) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_tablesService -> InsertAudit_tables", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertAudit_tables")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertAudit_tables.tablecode", audit_tables.TableCode)
	insertedId, err = r.audit_tablesRepo.InsertAudit_tables(ctx, audit_tables)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertAudit_tables.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateAudit_tables(ctx context.Context,audit_tables *model.Audit_tables, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_tablesService -> UpdateAudit_tables", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateAudit_tables")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateAudit_tables.id", id)
	tracker.AddParam("service.UpdateAudit_tables.tablecode", audit_tables.TableCode)

	err = r.audit_tablesRepo.UpdateAudit_tables(ctx, audit_tables, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateAudit_tables.updated", true)

	return nil
}

