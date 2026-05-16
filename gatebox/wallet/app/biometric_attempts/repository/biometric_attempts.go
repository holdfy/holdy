package biometric_attemptsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Biometric_attemptsRepositoryIF interface {
     GetBiometric_attempts(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetBiometric_attemptsById(ctx context.Context, id int64) (*model.Biometric_attempts, error)
     GetBiometric_attemptsByAttemptCode(ctx context.Context, attemptcode string) (*model.Biometric_attempts, error)
     InsertBiometric_attempts(ctx context.Context, biometric_attempts *model.Biometric_attempts) (int64, error)
     UpdateBiometric_attempts(ctx context.Context, biometric_attempts *model.Biometric_attempts, id int64) error
     DeleteBiometric_attemptsById(ctx context.Context, id int64) (bool, error)
}
 type Biometric_attemptsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewBiometric_attemptsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Biometric_attemptsRepository{
    return &Biometric_attemptsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Biometric_attempts"),
     }
}
func (t Biometric_attemptsRepository)  GetBiometric_attempts(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Biometric_attemptsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetBiometric_attempts")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetBiometric_attempts.offset", offset)
	tracker.AddParam("repository.GetBiometric_attempts.limit", limit)
	itemsPage 			= model.ItemsPage{}
	biometric_attemptss := []model.Biometric_attempts{}

	rows, err := t.PGRead.Query(ctx, SQL_BIOMETRIC_ATTEMPTS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Biometric_attemptsRepository.repository.GetBiometric_attemptss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var biometric_attempts model.Biometric_attempts
		err := rows.Scan(
			&biometric_attempts.ID,
			&biometric_attempts.AttemptCode,
			&biometric_attempts.UserId,
			&biometric_attempts.PalmHash,
			&biometric_attempts.AccuracyScore,
			&biometric_attempts.DeviceId,
			&biometric_attempts.IdAttemptResult,
			&biometric_attempts.IdFailureReason,
			&biometric_attempts.IpAddress,
			&biometric_attempts.UserAgent,
			&biometric_attempts.LocationData,
			&biometric_attempts.CreatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Biometric_attemptsRepository.repository.GetBiometric_attemptss.Scan: ", err.Error())
			return itemsPage, err
		}
		biometric_attemptss = append(biometric_attemptss, biometric_attempts)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Biometric_attemptsRepository.repository.GetBiometric_attemptss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(biometric_attemptss) > 0 {
		qtyRecords = biometric_attemptss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = biometric_attemptss

	tracker.AddResult("repository.GetBiometric_attempts.rows_returned", len(biometric_attemptss))
	tracker.AddResult("repository.GetBiometric_attempts.total_count", len(biometric_attemptss))

	return itemsPage, nil
}
func (t Biometric_attemptsRepository)  GetBiometric_attemptsById(ctx context.Context, id int64) (biometric_attempts *model.Biometric_attempts, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Biometric_attemptsRepository -> GetBiometric_attemptsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetBiometric_attemptsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetBiometric_attemptsById.id", id)

	biometric_attempts = new(model.Biometric_attempts)
	row := t.PGRead.QueryRow(ctx, SQL_GET_BIOMETRIC_ATTEMPTS_BY_ID, id)
		err = row.Scan(
			&biometric_attempts.ID,
			&biometric_attempts.AttemptCode,
			&biometric_attempts.UserId,
			&biometric_attempts.PalmHash,
			&biometric_attempts.AccuracyScore,
			&biometric_attempts.DeviceId,
			&biometric_attempts.IdAttemptResult,
			&biometric_attempts.IdFailureReason,
			&biometric_attempts.IpAddress,
			&biometric_attempts.UserAgent,
			&biometric_attempts.LocationData,
			&biometric_attempts.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Biometric_attemptsRepository.repository.GetBiometric_attemptsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetBiometric_attemptsById.found", true)
	return biometric_attempts, nil
}
func (t Biometric_attemptsRepository)  GetBiometric_attemptsByAttemptCode(ctx context.Context, attemptcode string) (biometric_attempts *model.Biometric_attempts, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Biometric_attemptsRepository -> GetBiometric_attemptsByAttemptCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetBiometric_attemptsByAttemptCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetBiometric_attemptsByAttemptCode.attemptcode", attemptcode)

	biometric_attempts = new(model.Biometric_attempts)
	row := t.PGRead.QueryRow(ctx, SQL_GET_BIOMETRIC_ATTEMPTS_BY_ATTEMPT_CODE, attemptcode)
		err = row.Scan(
			&biometric_attempts.ID,
			&biometric_attempts.AttemptCode,
			&biometric_attempts.UserId,
			&biometric_attempts.PalmHash,
			&biometric_attempts.AccuracyScore,
			&biometric_attempts.DeviceId,
			&biometric_attempts.IdAttemptResult,
			&biometric_attempts.IdFailureReason,
			&biometric_attempts.IpAddress,
			&biometric_attempts.UserAgent,
			&biometric_attempts.LocationData,
			&biometric_attempts.CreatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Biometric_attemptsRepository.repository.GetBiometric_attemptsByattemptcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return biometric_attempts, nil
}
func (t Biometric_attemptsRepository)  DeleteBiometric_attemptsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Biometric_attemptsRepository -> DeleteBiometric_attemptsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteBiometric_attemptsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_BIOMETRIC_ATTEMPTS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Biometric_attemptsRepository.repository.DeleteBiometric_attemptsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteBiometric_attemptsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteBiometric_attemptsById.deleted", result)
	return true, err
}
func (t Biometric_attemptsRepository)  InsertBiometric_attempts(ctx context.Context,biometric_attempts *model.Biometric_attempts) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Biometric_attemptsRepository -> InsertBiometric_attempts", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertBiometric_attempts")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertBiometric_attempts.attemptcode", biometric_attempts.AttemptCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_BIOMETRIC_ATTEMPTS_INSERT,
			biometric_attempts.AttemptCode,
			biometric_attempts.UserId,
			biometric_attempts.PalmHash,
			biometric_attempts.AccuracyScore,
			biometric_attempts.DeviceId,
			biometric_attempts.IdAttemptResult,
			biometric_attempts.IdFailureReason,
			biometric_attempts.IpAddress,
			biometric_attempts.UserAgent,
			biometric_attempts.LocationData,
			biometric_attempts.CreatedAt,
	).Scan(&biometric_attempts.ID)

	if err != nil {
		t.log.Error(ctx, "Biometric_attemptsRepository.repository.InsertBiometric_attempts.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertBiometric_attempts.inserted_id", biometric_attempts.ID)
   return biometric_attempts.ID, nil

}
func (t Biometric_attemptsRepository)  UpdateBiometric_attempts(ctx context.Context,biometric_attempts *model.Biometric_attempts, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Biometric_attemptsRepository -> UpdateBiometric_attempts", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateBiometric_attempts")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateBiometric_attempts.id", id)
	tracker.AddParam("repository.UpdateBiometric_attempts.attemptcode", biometric_attempts.AttemptCode)

	biometric_attempts.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_BIOMETRIC_ATTEMPTS_UPDATE, 
			biometric_attempts.AttemptCode,
			biometric_attempts.UserId,
			biometric_attempts.PalmHash,
			biometric_attempts.AccuracyScore,
			biometric_attempts.DeviceId,
			biometric_attempts.IdAttemptResult,
			biometric_attempts.IdFailureReason,
			biometric_attempts.IpAddress,
			biometric_attempts.UserAgent,
			biometric_attempts.LocationData,
			biometric_attempts.CreatedAt,
			biometric_attempts.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Biometric_attemptsRepository.repository.UpdateBiometric_attempts.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateBiometric_attempts.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateBiometric_attempts.rows_affected", rowsAffected)
	return nil
}

