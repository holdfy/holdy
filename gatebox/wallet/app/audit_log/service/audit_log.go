package audit_logSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	audit_logRepo "palm-pay/app/audit_log/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Audit_logServiceIF interface {
     GetAudit_log(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetAudit_logById(ctx context.Context, id int64) (*model.Audit_log, error)
     GetAudit_logByAuditCode(ctx context.Context, auditcode string) (*model.Audit_log, error)
     InsertAudit_log(ctx context.Context, audit_log *model.Audit_log) (int64, error)
     UpdateAudit_log(ctx context.Context, audit_log *model.Audit_log, id int64) error
     DeleteAudit_logById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     audit_logRepo audit_logRepo.Audit_logRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewAudit_logService(audit_logRepo audit_logRepo.Audit_logRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         audit_logRepo: audit_logRepo,
		  observability:  observabilidade.NewServiceObservability("service.audit_log"),
     }
}
func (r Resource)  GetAudit_log(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_logService -> GetAudit_log", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetAudit_log.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetAudit_log.offset", offset)
	tracker.AddParam("service.GetAudit_log.limit", limit)



	itemsPage, err = r.audit_logRepo.GetAudit_log(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Audit_log); ok {
		tracker.AddResult("service.GetAudit_log.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetAudit_logById(ctx context.Context, id int64) (audit_log *model.Audit_log, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_logService -> GetAudit_logById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetAudit_logById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetAudit_logById.id", id)
	audit_log, err = r.audit_logRepo.GetAudit_logById(ctx, id)
	if err != nil {
		return audit_log, errors.New(app.MsgRepositoryError)
	}

	return audit_log, nil
}
func (r Resource)  GetAudit_logByAuditCode(ctx context.Context, auditcode string) (audit_log *model.Audit_log, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_logService -> GetAudit_logByAuditCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetAudit_logByAuditCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetAudit_logByAuditCode.auditcode", auditcode)
	audit_log, err = r.audit_logRepo.GetAudit_logByAuditCode(ctx, auditcode)
	if err != nil {
		return audit_log, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetAudit_logByAuditCode.found", audit_log != nil)
	return audit_log, nil
}
func (r Resource)  DeleteAudit_logById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_logService -> DeleteAudit_logById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteAudit_logById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteAudit_logById.id",id)

	result, err = r.audit_logRepo.DeleteAudit_logById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteAudit_logById.deleted", result)
	return result, nil
}
func (r Resource)  InsertAudit_log(ctx context.Context,audit_log *model.Audit_log) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_logService -> InsertAudit_log", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertAudit_log")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertAudit_log.auditcode", audit_log.AuditCode)
	insertedId, err = r.audit_logRepo.InsertAudit_log(ctx, audit_log)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertAudit_log.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateAudit_log(ctx context.Context,audit_log *model.Audit_log, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_logService -> UpdateAudit_log", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateAudit_log")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateAudit_log.id", id)
	tracker.AddParam("service.UpdateAudit_log.auditcode", audit_log.AuditCode)

	err = r.audit_logRepo.UpdateAudit_log(ctx, audit_log, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateAudit_log.updated", true)

	return nil
}

