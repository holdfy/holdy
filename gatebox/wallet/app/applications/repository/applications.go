package applicationsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type ApplicationsRepositoryIF interface {
     GetApplications(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetApplicationsById(ctx context.Context, id int64) (*model.Applications, error)
     GetApplicationsByAppCode(ctx context.Context, appcode string) (*model.Applications, error)
     InsertApplications(ctx context.Context, applications *model.Applications) (int64, error)
     UpdateApplications(ctx context.Context, applications *model.Applications, id int64) error
     DeleteApplicationsById(ctx context.Context, id int64) (bool, error)
}
 type ApplicationsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewApplicationsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *ApplicationsRepository{
    return &ApplicationsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Applications"),
     }
}
func (t ApplicationsRepository)  GetApplications(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("ApplicationsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetApplications")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetApplications.offset", offset)
	tracker.AddParam("repository.GetApplications.limit", limit)
	itemsPage 			= model.ItemsPage{}
	applicationss := []model.Applications{}

	rows, err := t.PGRead.Query(ctx, SQL_APPLICATIONS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "ApplicationsRepository.repository.GetApplicationss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var applications model.Applications
		err := rows.Scan(
			&applications.ID,
			&applications.AppCode,
			&applications.Name,
			&applications.Code,
			&applications.Description,
			&applications.ApiKey,
			&applications.IsActive,
			&applications.Settings,
			&applications.CreatedAt,
			&applications.UpdatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "ApplicationsRepository.repository.GetApplicationss.Scan: ", err.Error())
			return itemsPage, err
		}
		applicationss = append(applicationss, applications)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "ApplicationsRepository.repository.GetApplicationss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(applicationss) > 0 {
		qtyRecords = applicationss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = applicationss

	tracker.AddResult("repository.GetApplications.rows_returned", len(applicationss))
	tracker.AddResult("repository.GetApplications.total_count", len(applicationss))

	return itemsPage, nil
}
func (t ApplicationsRepository)  GetApplicationsById(ctx context.Context, id int64) (applications *model.Applications, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("ApplicationsRepository -> GetApplicationsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetApplicationsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetApplicationsById.id", id)

	applications = new(model.Applications)
	row := t.PGRead.QueryRow(ctx, SQL_GET_APPLICATIONS_BY_ID, id)
		err = row.Scan(
			&applications.ID,
			&applications.AppCode,
			&applications.Name,
			&applications.Code,
			&applications.Description,
			&applications.ApiKey,
			&applications.IsActive,
			&applications.Settings,
			&applications.CreatedAt,
			&applications.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"ApplicationsRepository.repository.GetApplicationsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetApplicationsById.found", true)
	return applications, nil
}
func (t ApplicationsRepository)  GetApplicationsByAppCode(ctx context.Context, appcode string) (applications *model.Applications, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("ApplicationsRepository -> GetApplicationsByAppCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetApplicationsByAppCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetApplicationsByAppCode.appcode", appcode)

	applications = new(model.Applications)
	row := t.PGRead.QueryRow(ctx, SQL_GET_APPLICATIONS_BY_APP_CODE, appcode)
		err = row.Scan(
			&applications.ID,
			&applications.AppCode,
			&applications.Name,
			&applications.Code,
			&applications.Description,
			&applications.ApiKey,
			&applications.IsActive,
			&applications.Settings,
			&applications.CreatedAt,
			&applications.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"ApplicationsRepository.repository.GetApplicationsByappcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return applications, nil
}
func (t ApplicationsRepository)  DeleteApplicationsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("ApplicationsRepository -> DeleteApplicationsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteApplicationsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_APPLICATIONS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"ApplicationsRepository.repository.DeleteApplicationsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteApplicationsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteApplicationsById.deleted", result)
	return true, err
}
func (t ApplicationsRepository)  InsertApplications(ctx context.Context,applications *model.Applications) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("ApplicationsRepository -> InsertApplications", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertApplications")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertApplications.appcode", applications.AppCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_APPLICATIONS_INSERT,
			applications.AppCode,
			applications.Name,
			applications.Code,
			applications.Description,
			applications.ApiKey,
			applications.IsActive,
			applications.Settings,
			applications.CreatedAt,
			applications.UpdatedAt,
	).Scan(&applications.ID)

	if err != nil {
		t.log.Error(ctx, "ApplicationsRepository.repository.InsertApplications.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertApplications.inserted_id", applications.ID)
   return applications.ID, nil

}
func (t ApplicationsRepository)  UpdateApplications(ctx context.Context,applications *model.Applications, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("ApplicationsRepository -> UpdateApplications", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateApplications")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateApplications.id", id)
	tracker.AddParam("repository.UpdateApplications.appcode", applications.AppCode)

	applications.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_APPLICATIONS_UPDATE, 
			applications.AppCode,
			applications.Name,
			applications.Code,
			applications.Description,
			applications.ApiKey,
			applications.IsActive,
			applications.Settings,
			applications.CreatedAt,
			applications.UpdatedAt,
			applications.ID,
   )
	if err != nil {
		t.log.Error(ctx, "ApplicationsRepository.repository.UpdateApplications.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateApplications.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateApplications.rows_affected", rowsAffected)
	return nil
}

