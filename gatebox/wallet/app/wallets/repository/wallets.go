package walletsRepo

import (
  "palm-pay/model"
  "github.com/tungstenbyte/utils/logger"
  "time"
  "context"
  "errors"
  "github.com/jackc/pgx/v5/pgxpool"
  "palm-pay/utils/observabilidade"
)
 type WalletsRepositoryIF interface {
     GetWallets(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetWalletsById(ctx context.Context, id int64) (*model.Wallets, error)
     GetWalletsByWalletCode(ctx context.Context, walletcode string) (*model.Wallets, error)
     InsertWallets(ctx context.Context, wallets *model.Wallets) (int64, error)
     UpdateWallets(ctx context.Context, wallets *model.Wallets, id int64) error
     DeleteWalletsById(ctx context.Context, id int64) (bool, error)
}
 type WalletsRepository struct {
     PGRead  *pgxpool.Pool
     PGWrite *pgxpool.Pool
     log     logger.Logger
	  observability *observabilidade.RepositoryObservability
}
 func NewWalletsRepository(pgWrite *pgxpool.Pool, pgRead *pgxpool.Pool, log logger.Logger) *WalletsRepository{
    return &WalletsRepository{
         log:     log,
         PGWrite: pgWrite,
         PGRead:  pgRead,
		  observability: observabilidade.NewRepositoryObservability("Wallets"),
     }
}
func (t WalletsRepository)  GetWallets(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("WalletsRepository -> GetPermission", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetWallets")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetWallets.offset", offset)
	tracker.AddParam("repository.GetWallets.limit", limit)
	itemsPage 			= model.ItemsPage{}
	walletss := []model.Wallets{}

	rows, err := t.PGRead.Query(ctx, SQL_WALLETS_LIST, limit, offset)
	if err != nil {
		t.log.Error(ctx, "WalletsRepository.repository.GetWalletss.PG: ", err.Error())
		return itemsPage, err
	}

   defer rows.Close()

	for rows.Next() {
		var wallets model.Wallets
		err := rows.Scan(
			&wallets.ID,
			&wallets.WalletCode,
			&wallets.UserId,
			&wallets.ApplicationId,
			&wallets.Name,
			&wallets.IdWalletType,
			&wallets.IdCurrency,
			&wallets.IdStatus,
			&wallets.Balance,
			&wallets.AvailableBalance,
			&wallets.DailyLimit,
			&wallets.MonthlyLimit,
			&wallets.SignatureRequiredAbove,
			&wallets.IsPrimary,
			&wallets.Metadata,
			&wallets.CreatedAt,
			&wallets.UpdatedAt,
		)
		if err != nil {
			t.log.Error(ctx, "WalletsRepository.repository.GetWalletss.Scan: ", err.Error())
			return itemsPage, err
		}
		walletss = append(walletss, wallets)
	}
	if err = rows.Err(); err != nil { 
		t.log.Error(ctx, "WalletsRepository.repository.GetWalletss.Rows: ", err.Error())
		return itemsPage, err
	}

	qtyRecords := int64(0)
	if len(walletss) > 0 {
		qtyRecords = walletss[0].FullCount
	}

	itemsPage.Offset = offset
	itemsPage.Limit = limit
	itemsPage.Total = qtyRecords
	itemsPage.Items = walletss

	tracker.AddResult("repository.GetWallets.rows_returned", len(walletss))
	tracker.AddResult("repository.GetWallets.total_count", len(walletss))

	return itemsPage, nil
}
func (t WalletsRepository)  GetWalletsById(ctx context.Context, id int64) (wallets *model.Wallets, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("WalletsRepository -> GetWalletsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetWalletsById")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetWalletsById.id", id)

	wallets = new(model.Wallets)
	row := t.PGRead.QueryRow(ctx, SQL_GET_WALLETS_BY_ID, id)
		err = row.Scan(
			&wallets.ID,
			&wallets.WalletCode,
			&wallets.UserId,
			&wallets.ApplicationId,
			&wallets.Name,
			&wallets.IdWalletType,
			&wallets.IdCurrency,
			&wallets.IdStatus,
			&wallets.Balance,
			&wallets.AvailableBalance,
			&wallets.DailyLimit,
			&wallets.MonthlyLimit,
			&wallets.SignatureRequiredAbove,
			&wallets.IsPrimary,
			&wallets.Metadata,
			&wallets.CreatedAt,
			&wallets.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"WalletsRepository.repository.GetWalletsById: ", err.Error())
		return nil, err
	}
	tracker.AddResult("repository.GetWalletsById.found", true)
	return wallets, nil
}
func (t WalletsRepository)  GetWalletsByWalletCode(ctx context.Context, walletcode string) (wallets *model.Wallets, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("WalletsRepository -> GetWalletsByWalletCode", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "SELECT", "repository.GetWalletsByWalletCode")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.GetWalletsByWalletCode.walletcode", walletcode)

	wallets = new(model.Wallets)
	row := t.PGRead.QueryRow(ctx, SQL_GET_WALLETS_BY_WALLET_CODE, walletcode)
		err = row.Scan(
			&wallets.ID,
			&wallets.WalletCode,
			&wallets.UserId,
			&wallets.ApplicationId,
			&wallets.Name,
			&wallets.IdWalletType,
			&wallets.IdCurrency,
			&wallets.IdStatus,
			&wallets.Balance,
			&wallets.AvailableBalance,
			&wallets.DailyLimit,
			&wallets.MonthlyLimit,
			&wallets.SignatureRequiredAbove,
			&wallets.IsPrimary,
			&wallets.Metadata,
			&wallets.CreatedAt,
			&wallets.UpdatedAt,
		)
	if err != nil {
		t.log.Error(ctx,"WalletsRepository.repository.GetWalletsBywalletcode: ", err.Error())
		return nil, err
	}
	
	tracker.AddResult("found", true)
	return wallets, nil
}
func (t WalletsRepository)  DeleteWalletsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("WalletsRepository -> DeleteWalletsById", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "DELETE", "repository.DeleteWalletsById")
	defer tracker.Finish(&err)
	commandTag, err := t.PGWrite.Exec(ctx, SQL_WALLETS_DELETE_BY_ID, id)
	if err != nil {
		t.log.Error(ctx,"WalletsRepository.repository.DeleteWalletsById: ", err.Error())
		return false, err
	}

	if commandTag.RowsAffected() == 0 { 
		return false, nil
	}

	rowsAffected := commandTag.RowsAffected()
	result = rowsAffected > 0

	tracker.AddResult("repository.DeleteWalletsById.rows_affected", rowsAffected)
	tracker.AddResult("repository.DeleteWalletsById.deleted", result)
	return true, err
}
func (t WalletsRepository)  InsertWallets(ctx context.Context,wallets *model.Wallets) (insertedID int64, err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("WalletsRepository -> InsertWallets", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "INSERT", "repository.InsertWallets")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.InsertWallets.walletcode", wallets.WalletCode)

	err = t.PGWrite.QueryRow(ctx,  SQL_WALLETS_INSERT,
			wallets.WalletCode,
			wallets.UserId,
			wallets.ApplicationId,
			wallets.Name,
			wallets.IdWalletType,
			wallets.IdCurrency,
			wallets.IdStatus,
			wallets.Balance,
			wallets.AvailableBalance,
			wallets.DailyLimit,
			wallets.MonthlyLimit,
			wallets.SignatureRequiredAbove,
			wallets.IsPrimary,
			wallets.Metadata,
			wallets.CreatedAt,
			wallets.UpdatedAt,
	).Scan(&wallets.ID)

	if err != nil {
		t.log.Error(ctx, "WalletsRepository.repository.InsertWallets.PG: ", err.Error())
		return 0, err
	}


	tracker.AddResult("repository.InsertWallets.inserted_id", wallets.ID)
   return wallets.ID, nil

}
func (t WalletsRepository)  UpdateWallets(ctx context.Context,wallets *model.Wallets, id int64) (err error) {
	startedAt := time.Now()
	defer t.log.Chronometer("WalletsRepository -> UpdateWallets", &startedAt)

	tracker := t.observability.TrackQuery(ctx, "UPDATE", "repository.UpdateWallets")
	defer tracker.Finish(&err)

	tracker.AddParam("repository.UpdateWallets.id", id)
	tracker.AddParam("repository.UpdateWallets.walletcode", wallets.WalletCode)

	wallets.ID = id

	commandTag, err := t.PGWrite.Exec(ctx,SQL_WALLETS_UPDATE, 
			wallets.WalletCode,
			wallets.UserId,
			wallets.ApplicationId,
			wallets.Name,
			wallets.IdWalletType,
			wallets.IdCurrency,
			wallets.IdStatus,
			wallets.Balance,
			wallets.AvailableBalance,
			wallets.DailyLimit,
			wallets.MonthlyLimit,
			wallets.SignatureRequiredAbove,
			wallets.IsPrimary,
			wallets.Metadata,
			wallets.CreatedAt,
			wallets.UpdatedAt,
			wallets.ID,
   )
	if err != nil {
		t.log.Error(ctx, "WalletsRepository.repository.UpdateWallets.PG: ", err.Error())
		return err
	}

	rowsAffected := commandTag.RowsAffected()

	if rowsAffected == 0 {
		err := errors.New("no rows affected")
		t.log.Error(ctx, "CustomerRepository.repository.UpdateWallets.PG: ", err)
		return err
	}

	tracker.AddResult("repository.UpdateWallets.rows_affected", rowsAffected)
	return nil
}

