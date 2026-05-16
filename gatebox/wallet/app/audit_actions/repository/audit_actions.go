package audit_actionsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Audit_actionsRepositoryIF interface {
     GetAudit_actions(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetAudit_actionsById(ctx context.Context, id int64) (*model.Audit_actions, error)
     GetAudit_actionsByActionCode(ctx context.Context, actioncode string) (*model.Audit_actions, error)
     InsertAudit_actions(ctx context.Context, audit_actions *model.Audit_actions) (int64, error)
     UpdateAudit_actions(ctx context.Context, audit_actions *model.Audit_actions, id int64) error
     DeleteAudit_actionsById(ctx context.Context, id int64) (bool, error)
}
 type Audit_actionsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewAudit_actionsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Audit_actionsRepository{
    return &Audit_actionsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Audit_actions"),
     }
}
func (t Audit_actionsRepository)  GetAudit_actions(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_actionsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAudit_actions")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAudit_actions.offset", offset)
	tracker.AddParam("repository.GetAudit_actions.limit", limit)
	itemsPage 			= model.ItemsPage{}
	audit_actionss := []model.Audit_actions{}

	rows, err := t.PGRead.Query(ctx, SQL_AUDIT_ACTIONS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Audit_actionsRepository.repository.GetAudit_actionss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var audit_actions model.Audit_actions
		err := rows.Scan(
			&audit_actions.ID,
			&audit_actions.ActionCode,
			&audit_actions.Name,
			&audit_actions.Description,
			&audit_actions.SeverityLevel,
			&audit_actions.RequiresUser,
			&audit_actions.IsActive,
			&audit_actions.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Audit_actionsRepository.repository.GetAudit_actionss.Scan: ", err.Error())
			return itemsPage, err
		}
		audit_actionss = append(audit_actionss, audit_actions)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Audit_actionsRepository.repository.GetAudit_actionss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(audit_actionss) > 0 {
		qtyRecords = audit_actionss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = audit_actionss

	tracker.AddResult("repository.GetAudit_actions.rows_returned", len(audit_actionss))
	tracker.AddResult("repository.GetAudit_actions.total_count", len(audit_actionss))

	return itemsPage, nil
}
func (t Audit_actionsRepository)  GetAudit_actionsById(ctx context.Context, id int64) (audit_actions *model.Audit_actions, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_actionsRepository -> GetAudit_actionsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAudit_actionsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAudit_actionsById.id", id)

	audit_actions = new(model.Audit_actions)
	row := t.PGRead.QueryRow(ctx, SQL_GET_AUDIT_ACTIONS_BY_ID, id)
		err = row.Scan(
			&audit_actions.ID,
			&audit_actions.ActionCode,
			&audit_actions.Name,
			&audit_actions.Description,
			&audit_actions.SeverityLevel,
			&audit_actions.RequiresUser,
			&audit_actions.IsActive,
			&audit_actions.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Audit_actionsRepository.repository.GetAudit_actionsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetAudit_actionsById.found", true)
	return audit_actions, nil
}
func (t Audit_actionsRepository)  GetAudit_actionsByActionCode(ctx context.Context, actioncode string) (audit_actions *model.Audit_actions, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_actionsRepository -> GetAudit_actionsByActionCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAudit_actionsByActionCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAudit_actionsByActionCode.actioncode", actioncode)

	audit_actions = new(model.Audit_actions)
	row := t.PGRead.QueryRow(ctx, SQL_GET_AUDIT_ACTIONS_BY_ACTION_CODE, actioncode)
		err = row.Scan(
			&audit_actions.ID,
			&audit_actions.ActionCode,
			&audit_actions.Name,
			&audit_actions.Description,
			&audit_actions.SeverityLevel,
			&audit_actions.RequiresUser,
			&audit_actions.IsActive,
			&audit_actions.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Audit_actionsRepository.repository.GetAudit_actionsByactioncode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return audit_actions, nil
}
func (t Audit_actionsRepository)  DeleteAudit_actionsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_actionsRepository -> DeleteAudit_actionsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteAudit_actionsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_AUDIT_ACTIONS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Audit_actionsRepository.repository.DeleteAudit_actionsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteAudit_actionsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteAudit_actionsById.deleted", result)
	return true, err
}
func (t Audit_actionsRepository)  InsertAudit_actions(ctx context.Context,audit_actions *model.Audit_actions) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_actionsRepository -> InsertAudit_actions", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertAudit_actions")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertAudit_actions.actioncode", audit_actions.ActionCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_AUDIT_ACTIONS_INSERT,
			audit_actions.ActionCode,
			audit_actions.Name,
			audit_actions.Description,
			audit_actions.SeverityLevel,
			audit_actions.RequiresUser,
			audit_actions.IsActive,
			audit_actions.CreatedAt,
	).Scan(&audit_actions.ID)

	if err != nil {
		t.log.Error(ctx, "Audit_actionsRepository.repository.InsertAudit_actions.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertAudit_actions.inserted_id", audit_actions.ID)
   return audit_actions.ID, nil

}
func (t Audit_actionsRepository)  UpdateAudit_actions(ctx context.Context,audit_actions *model.Audit_actions, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_actionsRepository -> UpdateAudit_actions", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateAudit_actions")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateAudit_actions.id", id)
	tracker.AddParam("repository.UpdateAudit_actions.actioncode", audit_actions.ActionCode)

	audit_actions.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_AUDIT_ACTIONS_UPDATE, 
			audit_actions.ActionCode,
			audit_actions.Name,
			audit_actions.Description,
			audit_actions.SeverityLevel,
			audit_actions.RequiresUser,
			audit_actions.IsActive,
			audit_actions.CreatedAt,
			audit_actions.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Audit_actionsRepository.repository.UpdateAudit_actions.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateAudit_actions.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateAudit_actions.rows_affected", rowsAffected)
	return nil
}

