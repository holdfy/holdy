package external_walletsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type External_walletsRepositoryIF interface {
     GetExternal_wallets(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetExternal_walletsById(ctx context.Context, id int64) (*model.External_wallets, error)
     GetExternal_walletsByExternalWalletCode(ctx context.Context, externalwalletcode string) (*model.External_wallets, error)
     InsertExternal_wallets(ctx context.Context, external_wallets *model.External_wallets) (int64, error)
     UpdateExternal_wallets(ctx context.Context, external_wallets *model.External_wallets, id int64) error
     DeleteExternal_walletsById(ctx context.Context, id int64) (bool, error)
}
 type External_walletsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewExternal_walletsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *External_walletsRepository{
    return &External_walletsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("External_wallets"),
     }
}
func (t External_walletsRepository)  GetExternal_wallets(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("External_walletsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetExternal_wallets")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetExternal_wallets.offset", offset)
	tracker.AddParam("repository.GetExternal_wallets.limit", limit)
	itemsPage 			= model.ItemsPage{}
	external_walletss := []model.External_wallets{}

	rows, err := t.PGRead.Query(ctx, SQL_EXTERNAL_WALLETS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "External_walletsRepository.repository.GetExternal_walletss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var external_wallets model.External_wallets
		err := rows.Scan(
			&external_wallets.ID,
			&external_wallets.ExternalWalletCode,
			&external_wallets.UserId,
			&external_wallets.IdProvider,
			&external_wallets.ExternalAccountId,
			&external_wallets.AccountInfo,
			&external_wallets.AccessTokenEncrypted,
			&external_wallets.RefreshTokenEncrypted,
			&external_wallets.TokenExpiresAt,
			&external_wallets.IsActive,
			&external_wallets.LastSync,
			&external_wallets.CreatedAt,
			&external_wallets.UpdatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "External_walletsRepository.repository.GetExternal_walletss.Scan: ", err.Error())
			return itemsPage, err
		}
		external_walletss = append(external_walletss, external_wallets)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "External_walletsRepository.repository.GetExternal_walletss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(external_walletss) > 0 {
		qtyRecords = external_walletss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = external_walletss

	tracker.AddResult("repository.GetExternal_wallets.rows_returned", len(external_walletss))
	tracker.AddResult("repository.GetExternal_wallets.total_count", len(external_walletss))

	return itemsPage, nil
}
func (t External_walletsRepository)  GetExternal_walletsById(ctx context.Context, id int64) (external_wallets *model.External_wallets, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("External_walletsRepository -> GetExternal_walletsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetExternal_walletsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetExternal_walletsById.id", id)

	external_wallets = new(model.External_wallets)
	row := t.PGRead.QueryRow(ctx, SQL_GET_EXTERNAL_WALLETS_BY_ID, id)
		err = row.Scan(
			&external_wallets.ID,
			&external_wallets.ExternalWalletCode,
			&external_wallets.UserId,
			&external_wallets.IdProvider,
			&external_wallets.ExternalAccountId,
			&external_wallets.AccountInfo,
			&external_wallets.AccessTokenEncrypted,
			&external_wallets.RefreshTokenEncrypted,
			&external_wallets.TokenExpiresAt,
			&external_wallets.IsActive,
			&external_wallets.LastSync,
			&external_wallets.CreatedAt,
			&external_wallets.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"External_walletsRepository.repository.GetExternal_walletsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetExternal_walletsById.found", true)
	return external_wallets, nil
}
func (t External_walletsRepository)  GetExternal_walletsByExternalWalletCode(ctx context.Context, externalwalletcode string) (external_wallets *model.External_wallets, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("External_walletsRepository -> GetExternal_walletsByExternalWalletCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetExternal_walletsByExternalWalletCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetExternal_walletsByExternalWalletCode.externalwalletcode", externalwalletcode)

	external_wallets = new(model.External_wallets)
	row := t.PGRead.QueryRow(ctx, SQL_GET_EXTERNAL_WALLETS_BY_EXTERNAL_WALLET_CODE, externalwalletcode)
		err = row.Scan(
			&external_wallets.ID,
			&external_wallets.ExternalWalletCode,
			&external_wallets.UserId,
			&external_wallets.IdProvider,
			&external_wallets.ExternalAccountId,
			&external_wallets.AccountInfo,
			&external_wallets.AccessTokenEncrypted,
			&external_wallets.RefreshTokenEncrypted,
			&external_wallets.TokenExpiresAt,
			&external_wallets.IsActive,
			&external_wallets.LastSync,
			&external_wallets.CreatedAt,
			&external_wallets.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"External_walletsRepository.repository.GetExternal_walletsByexternalwalletcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return external_wallets, nil
}
func (t External_walletsRepository)  DeleteExternal_walletsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("External_walletsRepository -> DeleteExternal_walletsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteExternal_walletsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_EXTERNAL_WALLETS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"External_walletsRepository.repository.DeleteExternal_walletsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteExternal_walletsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteExternal_walletsById.deleted", result)
	return true, err
}
func (t External_walletsRepository)  InsertExternal_wallets(ctx context.Context,external_wallets *model.External_wallets) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("External_walletsRepository -> InsertExternal_wallets", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertExternal_wallets")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertExternal_wallets.externalwalletcode", external_wallets.ExternalWalletCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_EXTERNAL_WALLETS_INSERT,
			external_wallets.ExternalWalletCode,
			external_wallets.UserId,
			external_wallets.IdProvider,
			external_wallets.ExternalAccountId,
			external_wallets.AccountInfo,
			external_wallets.AccessTokenEncrypted,
			external_wallets.RefreshTokenEncrypted,
			external_wallets.TokenExpiresAt,
			external_wallets.IsActive,
			external_wallets.LastSync,
			external_wallets.CreatedAt,
			external_wallets.UpdatedAt,
	).Scan(&external_wallets.ID)

	if err != nil {
		t.log.Error(ctx, "External_walletsRepository.repository.InsertExternal_wallets.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertExternal_wallets.inserted_id", external_wallets.ID)
   return external_wallets.ID, nil

}
func (t External_walletsRepository)  UpdateExternal_wallets(ctx context.Context,external_wallets *model.External_wallets, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("External_walletsRepository -> UpdateExternal_wallets", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateExternal_wallets")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateExternal_wallets.id", id)
	tracker.AddParam("repository.UpdateExternal_wallets.externalwalletcode", external_wallets.ExternalWalletCode)

	external_wallets.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_EXTERNAL_WALLETS_UPDATE, 
			external_wallets.ExternalWalletCode,
			external_wallets.UserId,
			external_wallets.IdProvider,
			external_wallets.ExternalAccountId,
			external_wallets.AccountInfo,
			external_wallets.AccessTokenEncrypted,
			external_wallets.RefreshTokenEncrypted,
			external_wallets.TokenExpiresAt,
			external_wallets.IsActive,
			external_wallets.LastSync,
			external_wallets.CreatedAt,
			external_wallets.UpdatedAt,
			external_wallets.ID,
   )
	if err != nil {
		t.log.Error(ctx, "External_walletsRepository.repository.UpdateExternal_wallets.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateExternal_wallets.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateExternal_wallets.rows_affected", rowsAffected)
	return nil
}

