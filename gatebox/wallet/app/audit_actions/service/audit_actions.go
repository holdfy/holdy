package audit_actionsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	audit_actionsRepo "palm-pay/app/audit_actions/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Audit_actionsServiceIF interface {
     GetAudit_actions(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetAudit_actionsById(ctx context.Context, id int64) (*model.Audit_actions, error)
     GetAudit_actionsByActionCode(ctx context.Context, actioncode string) (*model.Audit_actions, error)
     InsertAudit_actions(ctx context.Context, audit_actions *model.Audit_actions) (int64, error)
     UpdateAudit_actions(ctx context.Context, audit_actions *model.Audit_actions, id int64) error
     DeleteAudit_actionsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     audit_actionsRepo audit_actionsRepo.Audit_actionsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewAudit_actionsService(audit_actionsRepo audit_actionsRepo.Audit_actionsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         audit_actionsRepo: audit_actionsRepo,
		  observability:  observabilidade.NewServiceObservability("service.audit_actions"),
     }
}
func (r Resource)  GetAudit_actions(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_actionsService -> GetAudit_actions", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetAudit_actions.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetAudit_actions.offset", offset)
	tracker.AddParam("service.GetAudit_actions.limit", limit)



	itemsPage, err = r.audit_actionsRepo.GetAudit_actions(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Audit_actions); ok {
		tracker.AddResult("service.GetAudit_actions.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetAudit_actionsById(ctx context.Context, id int64) (audit_actions *model.Audit_actions, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_actionsService -> GetAudit_actionsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetAudit_actionsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetAudit_actionsById.id", id)
	audit_actions, err = r.audit_actionsRepo.GetAudit_actionsById(ctx, id)
	if err != nil {
		return audit_actions, errors.New(app.MsgRepositoryError)
	}

	return audit_actions, nil
}
func (r Resource)  GetAudit_actionsByActionCode(ctx context.Context, actioncode string) (audit_actions *model.Audit_actions, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_actionsService -> GetAudit_actionsByActionCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetAudit_actionsByActionCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetAudit_actionsByActionCode.actioncode", actioncode)
	audit_actions, err = r.audit_actionsRepo.GetAudit_actionsByActionCode(ctx, actioncode)
	if err != nil {
		return audit_actions, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetAudit_actionsByActionCode.found", audit_actions != nil)
	return audit_actions, nil
}
func (r Resource)  DeleteAudit_actionsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_actionsService -> DeleteAudit_actionsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteAudit_actionsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteAudit_actionsById.id",id)

	result, err = r.audit_actionsRepo.DeleteAudit_actionsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteAudit_actionsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertAudit_actions(ctx context.Context,audit_actions *model.Audit_actions) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_actionsService -> InsertAudit_actions", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertAudit_actions")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertAudit_actions.actioncode", audit_actions.ActionCode)
	insertedId, err = r.audit_actionsRepo.InsertAudit_actions(ctx, audit_actions)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertAudit_actions.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateAudit_actions(ctx context.Context,audit_actions *model.Audit_actions, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Audit_actionsService -> UpdateAudit_actions", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateAudit_actions")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateAudit_actions.id", id)
	tracker.AddParam("service.UpdateAudit_actions.actioncode", audit_actions.ActionCode)

	err = r.audit_actionsRepo.UpdateAudit_actions(ctx, audit_actions, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateAudit_actions.updated", true)

	return nil
}

