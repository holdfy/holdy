package change_actorsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Change_actorsRepositoryIF interface {
     GetChange_actors(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetChange_actorsById(ctx context.Context, id int64) (*model.Change_actors, error)
     GetChange_actorsByActorCode(ctx context.Context, actorcode string) (*model.Change_actors, error)
     InsertChange_actors(ctx context.Context, change_actors *model.Change_actors) (int64, error)
     UpdateChange_actors(ctx context.Context, change_actors *model.Change_actors, id int64) error
     DeleteChange_actorsById(ctx context.Context, id int64) (bool, error)
}
 type Change_actorsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewChange_actorsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Change_actorsRepository{
    return &Change_actorsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Change_actors"),
     }
}
func (t Change_actorsRepository)  GetChange_actors(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Change_actorsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetChange_actors")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetChange_actors.offset", offset)
	tracker.AddParam("repository.GetChange_actors.limit", limit)
	itemsPage 			= model.ItemsPage{}
	change_actorss := []model.Change_actors{}

	rows, err := t.PGRead.Query(ctx, SQL_CHANGE_ACTORS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Change_actorsRepository.repository.GetChange_actorss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var change_actors model.Change_actors
		err := rows.Scan(
			&change_actors.ID,
			&change_actors.ActorCode,
			&change_actors.Name,
			&change_actors.Description,
			&change_actors.CanAutoApprove,
			&change_actors.PriorityLevel,
			&change_actors.IsActive,
			&change_actors.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Change_actorsRepository.repository.GetChange_actorss.Scan: ", err.Error())
			return itemsPage, err
		}
		change_actorss = append(change_actorss, change_actors)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Change_actorsRepository.repository.GetChange_actorss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(change_actorss) > 0 {
		qtyRecords = change_actorss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = change_actorss

	tracker.AddResult("repository.GetChange_actors.rows_returned", len(change_actorss))
	tracker.AddResult("repository.GetChange_actors.total_count", len(change_actorss))

	return itemsPage, nil
}
func (t Change_actorsRepository)  GetChange_actorsById(ctx context.Context, id int64) (change_actors *model.Change_actors, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Change_actorsRepository -> GetChange_actorsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetChange_actorsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetChange_actorsById.id", id)

	change_actors = new(model.Change_actors)
	row := t.PGRead.QueryRow(ctx, SQL_GET_CHANGE_ACTORS_BY_ID, id)
		err = row.Scan(
			&change_actors.ID,
			&change_actors.ActorCode,
			&change_actors.Name,
			&change_actors.Description,
			&change_actors.CanAutoApprove,
			&change_actors.PriorityLevel,
			&change_actors.IsActive,
			&change_actors.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Change_actorsRepository.repository.GetChange_actorsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetChange_actorsById.found", true)
	return change_actors, nil
}
func (t Change_actorsRepository)  GetChange_actorsByActorCode(ctx context.Context, actorcode string) (change_actors *model.Change_actors, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Change_actorsRepository -> GetChange_actorsByActorCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetChange_actorsByActorCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetChange_actorsByActorCode.actorcode", actorcode)

	change_actors = new(model.Change_actors)
	row := t.PGRead.QueryRow(ctx, SQL_GET_CHANGE_ACTORS_BY_ACTOR_CODE, actorcode)
		err = row.Scan(
			&change_actors.ID,
			&change_actors.ActorCode,
			&change_actors.Name,
			&change_actors.Description,
			&change_actors.CanAutoApprove,
			&change_actors.PriorityLevel,
			&change_actors.IsActive,
			&change_actors.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Change_actorsRepository.repository.GetChange_actorsByactorcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return change_actors, nil
}
func (t Change_actorsRepository)  DeleteChange_actorsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Change_actorsRepository -> DeleteChange_actorsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteChange_actorsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_CHANGE_ACTORS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Change_actorsRepository.repository.DeleteChange_actorsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteChange_actorsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteChange_actorsById.deleted", result)
	return true, err
}
func (t Change_actorsRepository)  InsertChange_actors(ctx context.Context,change_actors *model.Change_actors) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Change_actorsRepository -> InsertChange_actors", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertChange_actors")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertChange_actors.actorcode", change_actors.ActorCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_CHANGE_ACTORS_INSERT,
			change_actors.ActorCode,
			change_actors.Name,
			change_actors.Description,
			change_actors.CanAutoApprove,
			change_actors.PriorityLevel,
			change_actors.IsActive,
			change_actors.CreatedAt,
	).Scan(&change_actors.ID)

	if err != nil {
		t.log.Error(ctx, "Change_actorsRepository.repository.InsertChange_actors.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertChange_actors.inserted_id", change_actors.ID)
   return change_actors.ID, nil

}
func (t Change_actorsRepository)  UpdateChange_actors(ctx context.Context,change_actors *model.Change_actors, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Change_actorsRepository -> UpdateChange_actors", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateChange_actors")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateChange_actors.id", id)
	tracker.AddParam("repository.UpdateChange_actors.actorcode", change_actors.ActorCode)

	change_actors.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_CHANGE_ACTORS_UPDATE, 
			change_actors.ActorCode,
			change_actors.Name,
			change_actors.Description,
			change_actors.CanAutoApprove,
			change_actors.PriorityLevel,
			change_actors.IsActive,
			change_actors.CreatedAt,
			change_actors.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Change_actorsRepository.repository.UpdateChange_actors.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateChange_actors.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateChange_actors.rows_affected", rowsAffected)
	return nil
}

