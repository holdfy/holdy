package system_configurationsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type System_configurationsRepositoryIF interface {
     GetSystem_configurations(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetSystem_configurationsById(ctx context.Context, id int64) (*model.System_configurations, error)
     GetSystem_configurationsByConfigCode(ctx context.Context, configcode string) (*model.System_configurations, error)
     InsertSystem_configurations(ctx context.Context, system_configurations *model.System_configurations) (int64, error)
     UpdateSystem_configurations(ctx context.Context, system_configurations *model.System_configurations, id int64) error
     DeleteSystem_configurationsById(ctx context.Context, id int64) (bool, error)
}
 type System_configurationsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewSystem_configurationsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *System_configurationsRepository{
    return &System_configurationsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("System_configurations"),
     }
}
func (t System_configurationsRepository)  GetSystem_configurations(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("System_configurationsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSystem_configurations")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSystem_configurations.offset", offset)
	tracker.AddParam("repository.GetSystem_configurations.limit", limit)
	itemsPage 			= model.ItemsPage{}
	system_configurationss := []model.System_configurations{}

	rows, err := t.PGRead.Query(ctx, SQL_SYSTEM_CONFIGURATIONS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "System_configurationsRepository.repository.GetSystem_configurationss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var system_configurations model.System_configurations
		err := rows.Scan(
			&system_configurations.ID,
			&system_configurations.ConfigCode,
			&system_configurations.ApplicationId,
			&system_configurations.ConfigKey,
			&system_configurations.ConfigValue,
			&system_configurations.ConfigType,
			&system_configurations.Description,
			&system_configurations.IsActive,
			&system_configurations.CreatedAt,
			&system_configurations.UpdatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "System_configurationsRepository.repository.GetSystem_configurationss.Scan: ", err.Error())
			return itemsPage, err
		}
		system_configurationss = append(system_configurationss, system_configurations)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "System_configurationsRepository.repository.GetSystem_configurationss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(system_configurationss) > 0 {
		qtyRecords = system_configurationss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = system_configurationss

	tracker.AddResult("repository.GetSystem_configurations.rows_returned", len(system_configurationss))
	tracker.AddResult("repository.GetSystem_configurations.total_count", len(system_configurationss))

	return itemsPage, nil
}
func (t System_configurationsRepository)  GetSystem_configurationsById(ctx context.Context, id int64) (system_configurations *model.System_configurations, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("System_configurationsRepository -> GetSystem_configurationsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSystem_configurationsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSystem_configurationsById.id", id)

	system_configurations = new(model.System_configurations)
	row := t.PGRead.QueryRow(ctx, SQL_GET_SYSTEM_CONFIGURATIONS_BY_ID, id)
		err = row.Scan(
			&system_configurations.ID,
			&system_configurations.ConfigCode,
			&system_configurations.ApplicationId,
			&system_configurations.ConfigKey,
			&system_configurations.ConfigValue,
			&system_configurations.ConfigType,
			&system_configurations.Description,
			&system_configurations.IsActive,
			&system_configurations.CreatedAt,
			&system_configurations.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"System_configurationsRepository.repository.GetSystem_configurationsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetSystem_configurationsById.found", true)
	return system_configurations, nil
}
func (t System_configurationsRepository)  GetSystem_configurationsByConfigCode(ctx context.Context, configcode string) (system_configurations *model.System_configurations, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("System_configurationsRepository -> GetSystem_configurationsByConfigCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetSystem_configurationsByConfigCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetSystem_configurationsByConfigCode.configcode", configcode)

	system_configurations = new(model.System_configurations)
	row := t.PGRead.QueryRow(ctx, SQL_GET_SYSTEM_CONFIGURATIONS_BY_CONFIG_CODE, configcode)
		err = row.Scan(
			&system_configurations.ID,
			&system_configurations.ConfigCode,
			&system_configurations.ApplicationId,
			&system_configurations.ConfigKey,
			&system_configurations.ConfigValue,
			&system_configurations.ConfigType,
			&system_configurations.Description,
			&system_configurations.IsActive,
			&system_configurations.CreatedAt,
			&system_configurations.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"System_configurationsRepository.repository.GetSystem_configurationsByconfigcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return system_configurations, nil
}
func (t System_configurationsRepository)  DeleteSystem_configurationsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("System_configurationsRepository -> DeleteSystem_configurationsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteSystem_configurationsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_SYSTEM_CONFIGURATIONS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"System_configurationsRepository.repository.DeleteSystem_configurationsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteSystem_configurationsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteSystem_configurationsById.deleted", result)
	return true, err
}
func (t System_configurationsRepository)  InsertSystem_configurations(ctx context.Context,system_configurations *model.System_configurations) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("System_configurationsRepository -> InsertSystem_configurations", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertSystem_configurations")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertSystem_configurations.configcode", system_configurations.ConfigCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_SYSTEM_CONFIGURATIONS_INSERT,
			system_configurations.ConfigCode,
			system_configurations.ApplicationId,
			system_configurations.ConfigKey,
			system_configurations.ConfigValue,
			system_configurations.ConfigType,
			system_configurations.Description,
			system_configurations.IsActive,
			system_configurations.CreatedAt,
			system_configurations.UpdatedAt,
	).Scan(&system_configurations.ID)

	if err != nil {
		t.log.Error(ctx, "System_configurationsRepository.repository.InsertSystem_configurations.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertSystem_configurations.inserted_id", system_configurations.ID)
   return system_configurations.ID, nil

}
func (t System_configurationsRepository)  UpdateSystem_configurations(ctx context.Context,system_configurations *model.System_configurations, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("System_configurationsRepository -> UpdateSystem_configurations", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateSystem_configurations")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateSystem_configurations.id", id)
	tracker.AddParam("repository.UpdateSystem_configurations.configcode", system_configurations.ConfigCode)

	system_configurations.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_SYSTEM_CONFIGURATIONS_UPDATE, 
			system_configurations.ConfigCode,
			system_configurations.ApplicationId,
			system_configurations.ConfigKey,
			system_configurations.ConfigValue,
			system_configurations.ConfigType,
			system_configurations.Description,
			system_configurations.IsActive,
			system_configurations.CreatedAt,
			system_configurations.UpdatedAt,
			system_configurations.ID,
   )
	if err != nil {
		t.log.Error(ctx, "System_configurationsRepository.repository.UpdateSystem_configurations.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateSystem_configurations.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateSystem_configurations.rows_affected", rowsAffected)
	return nil
}

