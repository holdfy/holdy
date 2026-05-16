package notification_templatesRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Notification_templatesRepositoryIF interface {
     GetNotification_templates(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetNotification_templatesById(ctx context.Context, id int64) (*model.Notification_templates, error)
     GetNotification_templatesByTemplateCode(ctx context.Context, templatecode string) (*model.Notification_templates, error)
     InsertNotification_templates(ctx context.Context, notification_templates *model.Notification_templates) (int64, error)
     UpdateNotification_templates(ctx context.Context, notification_templates *model.Notification_templates, id int64) error
     DeleteNotification_templatesById(ctx context.Context, id int64) (bool, error)
}
 type Notification_templatesRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewNotification_templatesRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Notification_templatesRepository{
    return &Notification_templatesRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Notification_templates"),
     }
}
func (t Notification_templatesRepository)  GetNotification_templates(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_templatesRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetNotification_templates")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetNotification_templates.offset", offset)
	tracker.AddParam("repository.GetNotification_templates.limit", limit)
	itemsPage 			= model.ItemsPage{}
	notification_templatess := []model.Notification_templates{}

	rows, err := t.PGRead.Query(ctx, SQL_NOTIFICATION_TEMPLATES_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Notification_templatesRepository.repository.GetNotification_templatess.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var notification_templates model.Notification_templates
		err := rows.Scan(
			&notification_templates.ID,
			&notification_templates.TemplateCode,
			&notification_templates.ApplicationId,
			&notification_templates.TemplateKey,
			&notification_templates.IdChannel,
			&notification_templates.Subject,
			&notification_templates.TemplateBody,
			&notification_templates.TemplateVariables,
			&notification_templates.IsActive,
			&notification_templates.CreatedAt,
			&notification_templates.UpdatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Notification_templatesRepository.repository.GetNotification_templatess.Scan: ", err.Error())
			return itemsPage, err
		}
		notification_templatess = append(notification_templatess, notification_templates)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Notification_templatesRepository.repository.GetNotification_templatess.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(notification_templatess) > 0 {
		qtyRecords = notification_templatess[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = notification_templatess

	tracker.AddResult("repository.GetNotification_templates.rows_returned", len(notification_templatess))
	tracker.AddResult("repository.GetNotification_templates.total_count", len(notification_templatess))

	return itemsPage, nil
}
func (t Notification_templatesRepository)  GetNotification_templatesById(ctx context.Context, id int64) (notification_templates *model.Notification_templates, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_templatesRepository -> GetNotification_templatesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetNotification_templatesById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetNotification_templatesById.id", id)

	notification_templates = new(model.Notification_templates)
	row := t.PGRead.QueryRow(ctx, SQL_GET_NOTIFICATION_TEMPLATES_BY_ID, id)
		err = row.Scan(
			&notification_templates.ID,
			&notification_templates.TemplateCode,
			&notification_templates.ApplicationId,
			&notification_templates.TemplateKey,
			&notification_templates.IdChannel,
			&notification_templates.Subject,
			&notification_templates.TemplateBody,
			&notification_templates.TemplateVariables,
			&notification_templates.IsActive,
			&notification_templates.CreatedAt,
			&notification_templates.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Notification_templatesRepository.repository.GetNotification_templatesById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetNotification_templatesById.found", true)
	return notification_templates, nil
}
func (t Notification_templatesRepository)  GetNotification_templatesByTemplateCode(ctx context.Context, templatecode string) (notification_templates *model.Notification_templates, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_templatesRepository -> GetNotification_templatesByTemplateCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetNotification_templatesByTemplateCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetNotification_templatesByTemplateCode.templatecode", templatecode)

	notification_templates = new(model.Notification_templates)
	row := t.PGRead.QueryRow(ctx, SQL_GET_NOTIFICATION_TEMPLATES_BY_TEMPLATE_CODE, templatecode)
		err = row.Scan(
			&notification_templates.ID,
			&notification_templates.TemplateCode,
			&notification_templates.ApplicationId,
			&notification_templates.TemplateKey,
			&notification_templates.IdChannel,
			&notification_templates.Subject,
			&notification_templates.TemplateBody,
			&notification_templates.TemplateVariables,
			&notification_templates.IsActive,
			&notification_templates.CreatedAt,
			&notification_templates.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Notification_templatesRepository.repository.GetNotification_templatesBytemplatecode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return notification_templates, nil
}
func (t Notification_templatesRepository)  DeleteNotification_templatesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_templatesRepository -> DeleteNotification_templatesById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteNotification_templatesById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_NOTIFICATION_TEMPLATES_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Notification_templatesRepository.repository.DeleteNotification_templatesById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteNotification_templatesById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteNotification_templatesById.deleted", result)
	return true, err
}
func (t Notification_templatesRepository)  InsertNotification_templates(ctx context.Context,notification_templates *model.Notification_templates) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_templatesRepository -> InsertNotification_templates", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertNotification_templates")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertNotification_templates.templatecode", notification_templates.TemplateCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_NOTIFICATION_TEMPLATES_INSERT,
			notification_templates.TemplateCode,
			notification_templates.ApplicationId,
			notification_templates.TemplateKey,
			notification_templates.IdChannel,
			notification_templates.Subject,
			notification_templates.TemplateBody,
			notification_templates.TemplateVariables,
			notification_templates.IsActive,
			notification_templates.CreatedAt,
			notification_templates.UpdatedAt,
	).Scan(&notification_templates.ID)

	if err != nil {
		t.log.Error(ctx, "Notification_templatesRepository.repository.InsertNotification_templates.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertNotification_templates.inserted_id", notification_templates.ID)
   return notification_templates.ID, nil

}
func (t Notification_templatesRepository)  UpdateNotification_templates(ctx context.Context,notification_templates *model.Notification_templates, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Notification_templatesRepository -> UpdateNotification_templates", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateNotification_templates")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateNotification_templates.id", id)
	tracker.AddParam("repository.UpdateNotification_templates.templatecode", notification_templates.TemplateCode)

	notification_templates.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_NOTIFICATION_TEMPLATES_UPDATE, 
			notification_templates.TemplateCode,
			notification_templates.ApplicationId,
			notification_templates.TemplateKey,
			notification_templates.IdChannel,
			notification_templates.Subject,
			notification_templates.TemplateBody,
			notification_templates.TemplateVariables,
			notification_templates.IsActive,
			notification_templates.CreatedAt,
			notification_templates.UpdatedAt,
			notification_templates.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Notification_templatesRepository.repository.UpdateNotification_templates.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateNotification_templates.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateNotification_templates.rows_affected", rowsAffected)
	return nil
}

