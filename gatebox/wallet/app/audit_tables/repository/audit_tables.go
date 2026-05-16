package audit_tablesRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Audit_tablesRepositoryIF interface {
     GetAudit_tables(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetAudit_tablesById(ctx context.Context, id int64) (*model.Audit_tables, error)
     GetAudit_tablesByTableCode(ctx context.Context, tablecode string) (*model.Audit_tables, error)
     InsertAudit_tables(ctx context.Context, audit_tables *model.Audit_tables) (int64, error)
     UpdateAudit_tables(ctx context.Context, audit_tables *model.Audit_tables, id int64) error
     DeleteAudit_tablesById(ctx context.Context, id int64) (bool, error)
}
 type Audit_tablesRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewAudit_tablesRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Audit_tablesRepository{
    return &Audit_tablesRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Audit_tables"),
     }
}
func (t Audit_tablesRepository)  GetAudit_tables(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_tablesRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAudit_tables")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAudit_tables.offset", offset)
	tracker.AddParam("repository.GetAudit_tables.limit", limit)
	itemsPage 			= model.ItemsPage{}
	audit_tabless := []model.Audit_tables{}

	rows, err := t.PGRead.Query(ctx, SQL_AUDIT_TABLES_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Audit_tablesRepository.repository.GetAudit_tabless.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var audit_tables model.Audit_tables
		err := rows.Scan(
			&audit_tables.ID,
			&audit_tables.TableCode,
			&audit_tables.Name,
			&audit_tables.Description,
			&audit_tables.SensitivityLevel,
			&audit_tables.RetentionDays,
			&audit_tables.IsActive,
			&audit_tables.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Audit_tablesRepository.repository.GetAudit_tabless.Scan: ", err.Error())
			return itemsPage, err
		}
		audit_tabless = append(audit_tabless, audit_tables)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Audit_tablesRepository.repository.GetAudit_tabless.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(audit_tabless) > 0 {
		qtyRecords = audit_tabless[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = audit_tabless

	tracker.AddResult("repository.GetAudit_tables.rows_returned", len(audit_tabless))
	tracker.AddResult("repository.GetAudit_tables.total_count", len(audit_tabless))

	return itemsPage, nil
}
func (t Audit_tablesRepository)  GetAudit_tablesById(ctx context.Context, id int64) (audit_tables *model.Audit_tables, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_tablesRepository -> GetAudit_tablesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAudit_tablesById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAudit_tablesById.id", id)

	audit_tables = new(model.Audit_tables)
	row := t.PGRead.QueryRow(ctx, SQL_GET_AUDIT_TABLES_BY_ID, id)
		err = row.Scan(
			&audit_tables.ID,
			&audit_tables.TableCode,
			&audit_tables.Name,
			&audit_tables.Description,
			&audit_tables.SensitivityLevel,
			&audit_tables.RetentionDays,
			&audit_tables.IsActive,
			&audit_tables.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Audit_tablesRepository.repository.GetAudit_tablesById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetAudit_tablesById.found", true)
	return audit_tables, nil
}
func (t Audit_tablesRepository)  GetAudit_tablesByTableCode(ctx context.Context, tablecode string) (audit_tables *model.Audit_tables, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_tablesRepository -> GetAudit_tablesByTableCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetAudit_tablesByTableCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetAudit_tablesByTableCode.tablecode", tablecode)

	audit_tables = new(model.Audit_tables)
	row := t.PGRead.QueryRow(ctx, SQL_GET_AUDIT_TABLES_BY_TABLE_CODE, tablecode)
		err = row.Scan(
			&audit_tables.ID,
			&audit_tables.TableCode,
			&audit_tables.Name,
			&audit_tables.Description,
			&audit_tables.SensitivityLevel,
			&audit_tables.RetentionDays,
			&audit_tables.IsActive,
			&audit_tables.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Audit_tablesRepository.repository.GetAudit_tablesBytablecode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return audit_tables, nil
}
func (t Audit_tablesRepository)  DeleteAudit_tablesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_tablesRepository -> DeleteAudit_tablesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteAudit_tablesById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_AUDIT_TABLES_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Audit_tablesRepository.repository.DeleteAudit_tablesById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteAudit_tablesById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteAudit_tablesById.deleted", result)
	return true, err
}
func (t Audit_tablesRepository)  InsertAudit_tables(ctx context.Context,audit_tables *model.Audit_tables) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_tablesRepository -> InsertAudit_tables", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertAudit_tables")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertAudit_tables.tablecode", audit_tables.TableCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_AUDIT_TABLES_INSERT,
			audit_tables.TableCode,
			audit_tables.Name,
			audit_tables.Description,
			audit_tables.SensitivityLevel,
			audit_tables.RetentionDays,
			audit_tables.IsActive,
			audit_tables.CreatedAt,
	).Scan(&audit_tables.ID)

	if err != nil {
		t.log.Error(ctx, "Audit_tablesRepository.repository.InsertAudit_tables.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertAudit_tables.inserted_id", audit_tables.ID)
   return audit_tables.ID, nil

}
func (t Audit_tablesRepository)  UpdateAudit_tables(ctx context.Context,audit_tables *model.Audit_tables, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Audit_tablesRepository -> UpdateAudit_tables", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateAudit_tables")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateAudit_tables.id", id)
	tracker.AddParam("repository.UpdateAudit_tables.tablecode", audit_tables.TableCode)

	audit_tables.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_AUDIT_TABLES_UPDATE, 
			audit_tables.TableCode,
			audit_tables.Name,
			audit_tables.Description,
			audit_tables.SensitivityLevel,
			audit_tables.RetentionDays,
			audit_tables.IsActive,
			audit_tables.CreatedAt,
			audit_tables.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Audit_tablesRepository.repository.UpdateAudit_tables.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateAudit_tables.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateAudit_tables.rows_affected", rowsAffected)
	return nil
}

