package audit_logRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Audit_logRepositoryIF interface {
     GetAudit_log(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetAudit_logById(ctx context.Context, id int64) (*model.Audit_log, error)
     GetAudit_logByAuditCode(ctx context.Context, auditcode string) (*model.Audit_log, error)
     InsertAudit_log(ctx context.Context, audit_log *model.Audit_log) (int64, error)
     UpdateAudit_log(ctx context.Context, audit_log *model.Audit_log, id int64) error
     DeleteAudit_logById(ctx context.Context, id int64) (bool, error)
}
 type Audit_logRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewAudit_logRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Audit_logRepository{
    return &Audit_logRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Audit_log"),
     }
}
func (t Audit_logRepository)  GetAudit_log(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_logRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAudit_log")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAudit_log.offset", offset)
	tracker.AddParam("repository.GetAudit_log.limit", limit)
	itemsPage 			= model.ItemsPage{}
	audit_logs := []model.Audit_log{}

	rows, err := t.PGRead.Query(ctx, SQL_AUDIT_LOG_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Audit_logRepository.repository.GetAudit_logs.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var audit_log model.Audit_log
		err := rows.Scan(
			&audit_log.ID,
			&audit_log.AuditCode,
			&audit_log.IdTable,
			&audit_log.RecordId,
			&audit_log.IdAction,
			&audit_log.OldValues,
			&audit_log.NewValues,
			&audit_log.ChangedFields,
			&audit_log.UserId,
			&audit_log.ApplicationId,
			&audit_log.IpAddress,
			&audit_log.UserAgent,
			&audit_log.SessionId,
			&audit_log.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Audit_logRepository.repository.GetAudit_logs.Scan: ", err.Error())
			return itemsPage, err
		}
		audit_logs = append(audit_logs, audit_log)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Audit_logRepository.repository.GetAudit_logs.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(audit_logs) > 0 {
		qtyRecords = audit_logs[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = audit_logs

	tracker.AddResult("repository.GetAudit_log.rows_returned", len(audit_logs))
	tracker.AddResult("repository.GetAudit_log.total_count", len(audit_logs))

	return itemsPage, nil
}
func (t Audit_logRepository)  GetAudit_logById(ctx context.Context, id int64) (audit_log *model.Audit_log, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_logRepository -> GetAudit_logById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAudit_logById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAudit_logById.id", id)

	audit_log = new(model.Audit_log)
	row := t.PGRead.QueryRow(ctx, SQL_GET_AUDIT_LOG_BY_ID, id)
		err = row.Scan(
			&audit_log.ID,
			&audit_log.AuditCode,
			&audit_log.IdTable,
			&audit_log.RecordId,
			&audit_log.IdAction,
			&audit_log.OldValues,
			&audit_log.NewValues,
			&audit_log.ChangedFields,
			&audit_log.UserId,
			&audit_log.ApplicationId,
			&audit_log.IpAddress,
			&audit_log.UserAgent,
			&audit_log.SessionId,
			&audit_log.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Audit_logRepository.repository.GetAudit_logById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetAudit_logById.found", true)
	return audit_log, nil
}
func (t Audit_logRepository)  GetAudit_logByAuditCode(ctx context.Context, auditcode string) (audit_log *model.Audit_log, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_logRepository -> GetAudit_logByAuditCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAudit_logByAuditCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAudit_logByAuditCode.auditcode", auditcode)

	audit_log = new(model.Audit_log)
	row := t.PGRead.QueryRow(ctx, SQL_GET_AUDIT_LOG_BY_AUDIT_CODE, auditcode)
		err = row.Scan(
			&audit_log.ID,
			&audit_log.AuditCode,
			&audit_log.IdTable,
			&audit_log.RecordId,
			&audit_log.IdAction,
			&audit_log.OldValues,
			&audit_log.NewValues,
			&audit_log.ChangedFields,
			&audit_log.UserId,
			&audit_log.ApplicationId,
			&audit_log.IpAddress,
			&audit_log.UserAgent,
			&audit_log.SessionId,
			&audit_log.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Audit_logRepository.repository.GetAudit_logByauditcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return audit_log, nil
}
func (t Audit_logRepository)  DeleteAudit_logById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_logRepository -> DeleteAudit_logById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteAudit_logById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_AUDIT_LOG_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Audit_logRepository.repository.DeleteAudit_logById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteAudit_logById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteAudit_logById.deleted", result)
	return true, err
}
func (t Audit_logRepository)  InsertAudit_log(ctx context.Context,audit_log *model.Audit_log) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_logRepository -> InsertAudit_log", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertAudit_log")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertAudit_log.auditcode", audit_log.AuditCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_AUDIT_LOG_INSERT,
			audit_log.AuditCode,
			audit_log.IdTable,
			audit_log.RecordId,
			audit_log.IdAction,
			audit_log.OldValues,
			audit_log.NewValues,
			audit_log.ChangedFields,
			audit_log.UserId,
			audit_log.ApplicationId,
			audit_log.IpAddress,
			audit_log.UserAgent,
			audit_log.SessionId,
			audit_log.CreatedAt,
	).Scan(&audit_log.ID)

	if err != nil {
		t.log.Error(ctx, "Audit_logRepository.repository.InsertAudit_log.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertAudit_log.inserted_id", audit_log.ID)
   return audit_log.ID, nil

}
func (t Audit_logRepository)  UpdateAudit_log(ctx context.Context,audit_log *model.Audit_log, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_logRepository -> UpdateAudit_log", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateAudit_log")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateAudit_log.id", id)
	tracker.AddParam("repository.UpdateAudit_log.auditcode", audit_log.AuditCode)

	audit_log.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_AUDIT_LOG_UPDATE, 
			audit_log.AuditCode,
			audit_log.IdTable,
			audit_log.RecordId,
			audit_log.IdAction,
			audit_log.OldValues,
			audit_log.NewValues,
			audit_log.ChangedFields,
			audit_log.UserId,
			audit_log.ApplicationId,
			audit_log.IpAddress,
			audit_log.UserAgent,
			audit_log.SessionId,
			audit_log.CreatedAt,
			audit_log.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Audit_logRepository.repository.UpdateAudit_log.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateAudit_log.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateAudit_log.rows_affected", rowsAffected)
	return nil
}

