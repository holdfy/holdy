package security_severity_levelsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Security_severity_levelsRepositoryIF interface {
     GetSecurity_severity_levels(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetSecurity_severity_levelsById(ctx context.Context, id int64) (*model.Security_severity_levels, error)
     GetSecurity_severity_levelsBySeverityCode(ctx context.Context, severitycode string) (*model.Security_severity_levels, error)
     InsertSecurity_severity_levels(ctx context.Context, security_severity_levels *model.Security_severity_levels) (int64, error)
     UpdateSecurity_severity_levels(ctx context.Context, security_severity_levels *model.Security_severity_levels, id int64) error
     DeleteSecurity_severity_levelsById(ctx context.Context, id int64) (bool, error)
}
 type Security_severity_levelsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewSecurity_severity_levelsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Security_severity_levelsRepository{
    return &Security_severity_levelsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Security_severity_levels"),
     }
}
func (t Security_severity_levelsRepository)  GetSecurity_severity_levels(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_severity_levelsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSecurity_severity_levels")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSecurity_severity_levels.offset", offset)
	tracker.AddParam("repository.GetSecurity_severity_levels.limit", limit)
	itemsPage 			= model.ItemsPage{}
	security_severity_levelss := []model.Security_severity_levels{}

	rows, err := t.PGRead.Query(ctx, SQL_SECURITY_SEVERITY_LEVELS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Security_severity_levelsRepository.repository.GetSecurity_severity_levelss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var security_severity_levels model.Security_severity_levels
		err := rows.Scan(
			&security_severity_levels.ID,
			&security_severity_levels.SeverityCode,
			&security_severity_levels.Name,
			&security_severity_levels.Description,
			&security_severity_levels.LevelNumber,
			&security_severity_levels.NotificationRequired,
			&security_severity_levels.EscalationRequired,
			&security_severity_levels.IsActive,
			&security_severity_levels.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Security_severity_levelsRepository.repository.GetSecurity_severity_levelss.Scan: ", err.Error())
			return itemsPage, err
		}
		security_severity_levelss = append(security_severity_levelss, security_severity_levels)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Security_severity_levelsRepository.repository.GetSecurity_severity_levelss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(security_severity_levelss) > 0 {
		qtyRecords = security_severity_levelss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = security_severity_levelss

	tracker.AddResult("repository.GetSecurity_severity_levels.rows_returned", len(security_severity_levelss))
	tracker.AddResult("repository.GetSecurity_severity_levels.total_count", len(security_severity_levelss))

	return itemsPage, nil
}
func (t Security_severity_levelsRepository)  GetSecurity_severity_levelsById(ctx context.Context, id int64) (security_severity_levels *model.Security_severity_levels, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_severity_levelsRepository -> GetSecurity_severity_levelsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSecurity_severity_levelsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSecurity_severity_levelsById.id", id)

	security_severity_levels = new(model.Security_severity_levels)
	row := t.PGRead.QueryRow(ctx, SQL_GET_SECURITY_SEVERITY_LEVELS_BY_ID, id)
		err = row.Scan(
			&security_severity_levels.ID,
			&security_severity_levels.SeverityCode,
			&security_severity_levels.Name,
			&security_severity_levels.Description,
			&security_severity_levels.LevelNumber,
			&security_severity_levels.NotificationRequired,
			&security_severity_levels.EscalationRequired,
			&security_severity_levels.IsActive,
			&security_severity_levels.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Security_severity_levelsRepository.repository.GetSecurity_severity_levelsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetSecurity_severity_levelsById.found", true)
	return security_severity_levels, nil
}
func (t Security_severity_levelsRepository)  GetSecurity_severity_levelsBySeverityCode(ctx context.Context, severitycode string) (security_severity_levels *model.Security_severity_levels, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_severity_levelsRepository -> GetSecurity_severity_levelsBySeverityCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSecurity_severity_levelsBySeverityCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSecurity_severity_levelsBySeverityCode.severitycode", severitycode)

	security_severity_levels = new(model.Security_severity_levels)
	row := t.PGRead.QueryRow(ctx, SQL_GET_SECURITY_SEVERITY_LEVELS_BY_SEVERITY_CODE, severitycode)
		err = row.Scan(
			&security_severity_levels.ID,
			&security_severity_levels.SeverityCode,
			&security_severity_levels.Name,
			&security_severity_levels.Description,
			&security_severity_levels.LevelNumber,
			&security_severity_levels.NotificationRequired,
			&security_severity_levels.EscalationRequired,
			&security_severity_levels.IsActive,
			&security_severity_levels.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Security_severity_levelsRepository.repository.GetSecurity_severity_levelsByseveritycode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return security_severity_levels, nil
}
func (t Security_severity_levelsRepository)  DeleteSecurity_severity_levelsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_severity_levelsRepository -> DeleteSecurity_severity_levelsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteSecurity_severity_levelsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_SECURITY_SEVERITY_LEVELS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Security_severity_levelsRepository.repository.DeleteSecurity_severity_levelsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteSecurity_severity_levelsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteSecurity_severity_levelsById.deleted", result)
	return true, err
}
func (t Security_severity_levelsRepository)  InsertSecurity_severity_levels(ctx context.Context,security_severity_levels *model.Security_severity_levels) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_severity_levelsRepository -> InsertSecurity_severity_levels", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertSecurity_severity_levels")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertSecurity_severity_levels.severitycode", security_severity_levels.SeverityCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_SECURITY_SEVERITY_LEVELS_INSERT,
			security_severity_levels.SeverityCode,
			security_severity_levels.Name,
			security_severity_levels.Description,
			security_severity_levels.LevelNumber,
			security_severity_levels.NotificationRequired,
			security_severity_levels.EscalationRequired,
			security_severity_levels.IsActive,
			security_severity_levels.CreatedAt,
	).Scan(&security_severity_levels.ID)

	if err != nil {
		t.log.Error(ctx, "Security_severity_levelsRepository.repository.InsertSecurity_severity_levels.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertSecurity_severity_levels.inserted_id", security_severity_levels.ID)
   return security_severity_levels.ID, nil

}
func (t Security_severity_levelsRepository)  UpdateSecurity_severity_levels(ctx context.Context,security_severity_levels *model.Security_severity_levels, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Security_severity_levelsRepository -> UpdateSecurity_severity_levels", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateSecurity_severity_levels")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateSecurity_severity_levels.id", id)
	tracker.AddParam("repository.UpdateSecurity_severity_levels.severitycode", security_severity_levels.SeverityCode)

	security_severity_levels.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_SECURITY_SEVERITY_LEVELS_UPDATE, 
			security_severity_levels.SeverityCode,
			security_severity_levels.Name,
			security_severity_levels.Description,
			security_severity_levels.LevelNumber,
			security_severity_levels.NotificationRequired,
			security_severity_levels.EscalationRequired,
			security_severity_levels.IsActive,
			security_severity_levels.CreatedAt,
			security_severity_levels.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Security_severity_levelsRepository.repository.UpdateSecurity_severity_levels.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateSecurity_severity_levels.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateSecurity_severity_levels.rows_affected", rowsAffected)
	return nil
}

