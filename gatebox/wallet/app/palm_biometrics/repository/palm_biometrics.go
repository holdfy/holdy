package palm_biometricsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type Palm_biometricsRepositoryIF interface {
     GetPalm_biometrics(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetPalm_biometricsById(ctx context.Context, id int64) (*model.Palm_biometrics, error)
     GetPalm_biometricsByBiometricCode(ctx context.Context, biometriccode string) (*model.Palm_biometrics, error)
     InsertPalm_biometrics(ctx context.Context, palm_biometrics *model.Palm_biometrics) (int64, error)
     UpdatePalm_biometrics(ctx context.Context, palm_biometrics *model.Palm_biometrics, id int64) error
     DeletePalm_biometricsById(ctx context.Context, id int64) (bool, error)
}
 type Palm_biometricsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewPalm_biometricsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *Palm_biometricsRepository{
    return &Palm_biometricsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Palm_biometrics"),
     }
}
func (t Palm_biometricsRepository)  GetPalm_biometrics(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Palm_biometricsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetPalm_biometrics")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetPalm_biometrics.offset", offset)
	tracker.AddParam("repository.GetPalm_biometrics.limit", limit)
	itemsPage 			= model.ItemsPage{}
	palm_biometricss := []model.Palm_biometrics{}

	rows, err := t.PGRead.Query(ctx, SQL_PALM_BIOMETRICS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "Palm_biometricsRepository.repository.GetPalm_biometricss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var palm_biometrics model.Palm_biometrics
		err := rows.Scan(
			&palm_biometrics.ID,
			&palm_biometrics.BiometricCode,
			&palm_biometrics.UserId,
			&palm_biometrics.PalmHash,
			&palm_biometrics.AccuracyScore,
			&palm_biometrics.IdHandType,
			&palm_biometrics.EnrollmentDeviceId,
			&palm_biometrics.BitmapSignature,
			&palm_biometrics.IsPrimary,
			&palm_biometrics.IsActive,
			&palm_biometrics.RegisteredAt,
			&palm_biometrics.LastUsed,
			&palm_biometrics.UsageCount,
			&palm_biometrics.CreatedAt,
			&palm_biometrics.UpdatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "Palm_biometricsRepository.repository.GetPalm_biometricss.Scan: ", err.Error())
			return itemsPage, err
		}
		palm_biometricss = append(palm_biometricss, palm_biometrics)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "Palm_biometricsRepository.repository.GetPalm_biometricss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(palm_biometricss) > 0 {
		qtyRecords = palm_biometricss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = palm_biometricss

	tracker.AddResult("repository.GetPalm_biometrics.rows_returned", len(palm_biometricss))
	tracker.AddResult("repository.GetPalm_biometrics.total_count", len(palm_biometricss))

	return itemsPage, nil
}
func (t Palm_biometricsRepository)  GetPalm_biometricsById(ctx context.Context, id int64) (palm_biometrics *model.Palm_biometrics, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Palm_biometricsRepository -> GetPalm_biometricsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetPalm_biometricsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetPalm_biometricsById.id", id)

	palm_biometrics = new(model.Palm_biometrics)
	row := t.PGRead.QueryRow(ctx, SQL_GET_PALM_BIOMETRICS_BY_ID, id)
		err = row.Scan(
			&palm_biometrics.ID,
			&palm_biometrics.BiometricCode,
			&palm_biometrics.UserId,
			&palm_biometrics.PalmHash,
			&palm_biometrics.AccuracyScore,
			&palm_biometrics.IdHandType,
			&palm_biometrics.EnrollmentDeviceId,
			&palm_biometrics.BitmapSignature,
			&palm_biometrics.IsPrimary,
			&palm_biometrics.IsActive,
			&palm_biometrics.RegisteredAt,
			&palm_biometrics.LastUsed,
			&palm_biometrics.UsageCount,
			&palm_biometrics.CreatedAt,
			&palm_biometrics.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Palm_biometricsRepository.repository.GetPalm_biometricsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetPalm_biometricsById.found", true)
	return palm_biometrics, nil
}
func (t Palm_biometricsRepository)  GetPalm_biometricsByBiometricCode(ctx context.Context, biometriccode string) (palm_biometrics *model.Palm_biometrics, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Palm_biometricsRepository -> GetPalm_biometricsByBiometricCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetPalm_biometricsByBiometricCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetPalm_biometricsByBiometricCode.biometriccode", biometriccode)

	palm_biometrics = new(model.Palm_biometrics)
	row := t.PGRead.QueryRow(ctx, SQL_GET_PALM_BIOMETRICS_BY_BIOMETRIC_CODE, biometriccode)
		err = row.Scan(
			&palm_biometrics.ID,
			&palm_biometrics.BiometricCode,
			&palm_biometrics.UserId,
			&palm_biometrics.PalmHash,
			&palm_biometrics.AccuracyScore,
			&palm_biometrics.IdHandType,
			&palm_biometrics.EnrollmentDeviceId,
			&palm_biometrics.BitmapSignature,
			&palm_biometrics.IsPrimary,
			&palm_biometrics.IsActive,
			&palm_biometrics.RegisteredAt,
			&palm_biometrics.LastUsed,
			&palm_biometrics.UsageCount,
			&palm_biometrics.CreatedAt,
			&palm_biometrics.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"Palm_biometricsRepository.repository.GetPalm_biometricsBybiometriccode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return palm_biometrics, nil
}
func (t Palm_biometricsRepository)  DeletePalm_biometricsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Palm_biometricsRepository -> DeletePalm_biometricsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeletePalm_biometricsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_PALM_BIOMETRICS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"Palm_biometricsRepository.repository.DeletePalm_biometricsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeletePalm_biometricsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeletePalm_biometricsById.deleted", result)
	return true, err
}
func (t Palm_biometricsRepository)  InsertPalm_biometrics(ctx context.Context,palm_biometrics *model.Palm_biometrics) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Palm_biometricsRepository -> InsertPalm_biometrics", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertPalm_biometrics")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertPalm_biometrics.biometriccode", palm_biometrics.BiometricCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_PALM_BIOMETRICS_INSERT,
			palm_biometrics.BiometricCode,
			palm_biometrics.UserId,
			palm_biometrics.PalmHash,
			palm_biometrics.AccuracyScore,
			palm_biometrics.IdHandType,
			palm_biometrics.EnrollmentDeviceId,
			palm_biometrics.BitmapSignature,
			palm_biometrics.IsPrimary,
			palm_biometrics.IsActive,
			palm_biometrics.RegisteredAt,
			palm_biometrics.LastUsed,
			palm_biometrics.UsageCount,
			palm_biometrics.CreatedAt,
			palm_biometrics.UpdatedAt,
	).Scan(&palm_biometrics.ID)

	if err != nil {
		t.log.Error(ctx, "Palm_biometricsRepository.repository.InsertPalm_biometrics.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertPalm_biometrics.inserted_id", palm_biometrics.ID)
   return palm_biometrics.ID, nil

}
func (t Palm_biometricsRepository)  UpdatePalm_biometrics(ctx context.Context,palm_biometrics *model.Palm_biometrics, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("Palm_biometricsRepository -> UpdatePalm_biometrics", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdatePalm_biometrics")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdatePalm_biometrics.id", id)
	tracker.AddParam("repository.UpdatePalm_biometrics.biometriccode", palm_biometrics.BiometricCode)

	palm_biometrics.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_PALM_BIOMETRICS_UPDATE, 
			palm_biometrics.BiometricCode,
			palm_biometrics.UserId,
			palm_biometrics.PalmHash,
			palm_biometrics.AccuracyScore,
			palm_biometrics.IdHandType,
			palm_biometrics.EnrollmentDeviceId,
			palm_biometrics.BitmapSignature,
			palm_biometrics.IsPrimary,
			palm_biometrics.IsActive,
			palm_biometrics.RegisteredAt,
			palm_biometrics.LastUsed,
			palm_biometrics.UsageCount,
			palm_biometrics.CreatedAt,
			palm_biometrics.UpdatedAt,
			palm_biometrics.ID,
   )
	if err != nil {
		t.log.Error(ctx, "Palm_biometricsRepository.repository.UpdatePalm_biometrics.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdatePalm_biometrics.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdatePalm_biometrics.rows_affected", rowsAffected)
	return nil
}

